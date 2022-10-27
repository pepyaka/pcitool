//!  The /sys filesystem on Linux 2.6 and newer. The standard header of the config space is
//!  available to all users, the  rest  only  to  root.  Supports  extended configuration space,
//!  PCI domains, VPD (from Linux 2.6.26), physical slots (also since Linux 2.6.26) and information
//!  on attached kernel drivers.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use thiserror::Error;
use walkdir::WalkDir;

use super::{Access, AccessError, AccessMethod};
use crate::device::{Address, ConfigurationSpace, Device};

mod modules_alias;
use modules_alias::ModulesAlias;

mod slots;
use slots::Slots;

#[derive(Debug, Error)]
pub enum LinuxSysfsError {
    #[error("{path} read problem")]
    ReadDir { path: PathBuf, source: io::Error },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxSysfs {
    sysfs_path: PathBuf,
    modules_alias: Option<ModulesAlias>,
    slots: Option<Slots>,
}

impl LinuxSysfs {
    pub const PATH: &'static str = "/sys/bus/pci";
    pub fn new(sysfs_path: impl Into<PathBuf>) -> Self {
        let sysfs_path = sysfs_path.into();
        let modules_alias = uname::uname()
            .and_then(|info| {
                let path = Path::new("/lib/modules")
                    .join(info.release)
                    .join("modules.alias");

                ModulesAlias::init(path)
            })
            .ok();
        let slots = Slots::init(sysfs_path.join("slots")).ok();
        Self {
            sysfs_path,
            modules_alias,
            slots,
        }
    }
    pub fn modules_alias(&mut self, modules_alias: impl Into<ModulesAlias>) -> &mut Self {
        self.modules_alias = Some(modules_alias.into());
        self
    }
    pub fn slots(&mut self, slots: impl Into<Slots>) -> &mut Self {
        self.slots = Some(slots.into());
        self
    }
    pub fn access(&self) -> super::Result<Access> {
        // Check directory
        let is_dir = fs::metadata(&self.sysfs_path)
            .map_err(|source| AccessError::File {
                path: self.sysfs_path.clone(),
                source,
            })?
            .is_dir();
        if !is_dir {
            return Err(AccessError::File {
                path: self.sysfs_path.clone(),
                source: io::Error::new(io::ErrorKind::Other, "is not a directory"),
            });
        }
        Ok(Access::LinuxSysfs(self.clone()))
    }
    fn dev_dir_entries(&self) -> walkdir::IntoIter {
        WalkDir::new(&self.sysfs_path.join("devices"))
            .min_depth(1)
            .max_depth(1)
            .follow_links(true)
            .into_iter()
    }
    fn read_device(
        sysfs_path: impl Into<PathBuf>,
        modules_alias: &Option<ModulesAlias>,
        slots: &Option<Slots>,
    ) -> super::Result<Device> {
        let path = sysfs_path.into();
        let address = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(AccessError::File {
                path: path.clone(),
                source: io::Error::new(io::ErrorKind::Other, "file name unredable"),
            })?
            .to_string();
        let address: Address = address
            .parse()
            .map_err(|source| AccessError::ParseAddress { address, source })?;
        let config_path = path.join("config");
        let bytes = fs::read(&config_path).map_err(|source| AccessError::File {
            path: config_path,
            source,
        })?;
        let mut device = bytes
            .as_slice()
            .try_into()
            .map(|cs: ConfigurationSpace| Device::new(address.clone(), cs))
            .map_err(|_| AccessError::ConfigurationSpace)?;
        let label_path = path.join("label");
        device.label = fs::read_to_string(&label_path)
            .map_err(|err| {
                if err.kind() != io::ErrorKind::NotFound {
                    eprintln!(
                        "access::linux_sysfs: Error reading {}: {}",
                        label_path.display(),
                        err
                    );
                }
            })
            .ok();
        device.phy_slot = slots.as_ref().and_then(|slots| {
            slots.find(Address {
                function: 0,
                ..address
            })
        });
        device.numa_node = fs::read_to_string(path.join("numa_node"))
            .ok()
            .and_then(|s| u16::from_str_radix(s.trim(), 16).ok());
        device.iommu_group = fs::read_to_string(path.join("iommu_group")).ok();
        device.irq = fs::read_to_string(path.join("irq"))
            .ok()
            .and_then(|s| s.trim().parse().ok());
        device.resource = fs::read_to_string(path.join("resource"))
            .ok()
            .and_then(|s| s.parse().ok());

        device.driver_in_use = fs::read_link(path.join("driver"))
            .ok()
            .and_then(|path| path.file_name()?.to_str().map(|s| s.to_string()));
        device.kernel_modules =
            fs::read_to_string(path.join("modalias"))
                .ok()
                .and_then(|modalias| {
                    let mut kernel_modules = modules_alias
                        .as_ref()?
                        .lookup(&modalias)
                        .collect::<Vec<_>>();
                    kernel_modules.dedup();
                    Some(kernel_modules)
                });
        Ok(device)
    }
}

impl Default for LinuxSysfs {
    fn default() -> Self {
        Self::new(Self::PATH)
    }
}

impl<'a> AccessMethod<'a> for LinuxSysfs {
    type Scan = Scan;
    type Iter = Iter<'a>;
    fn device(&self, address: Address) -> super::Result<Device> {
        let sysfs_path = self.sysfs_path.join("devices").join(address.to_string());
        Self::read_device(sysfs_path, &self.modules_alias, &self.slots)
    }
    fn scan(&'a self) -> Self::Scan {
        Scan::new(self.dev_dir_entries())
    }
    fn iter(&'a self) -> Self::Iter {
        Iter::new(self.dev_dir_entries(), &self.modules_alias, &self.slots)
    }
    fn vital_product_data(&'a self, addr: Address) -> io::Result<Vec<u8>> {
        let path = self
            .sysfs_path
            .join("devices")
            .join(addr.to_string())
            .join("vpd");
        fs::read(path)
    }
}

#[derive(Debug)]
pub struct Scan {
    iter: walkdir::IntoIter,
}

impl Scan {
    pub fn new(iter: walkdir::IntoIter) -> Self {
        Self { iter }
    }
}

impl Iterator for Scan {
    type Item = super::Result<Address>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.iter.next()?;
        let result = entry
            .map(|entry| entry.file_name().to_string_lossy().into())
            .map_err(|source| {
                let path = source
                    .path()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| "unknown".into());
                let source = source
                    .into_io_error()
                    .unwrap_or_else(|| io::ErrorKind::Other.into());
                AccessError::File { path, source }
            })
            .and_then(|address: String| {
                address
                    .parse()
                    .map_err(|source| AccessError::ParseAddress { address, source })
            });
        Some(result)
    }
}

#[derive(Debug)]
pub struct Iter<'a> {
    iter: walkdir::IntoIter,
    modules_alias: &'a Option<ModulesAlias>,
    slots: &'a Option<Slots>,
}

impl<'a> Iter<'a> {
    pub fn new(
        iter: walkdir::IntoIter,
        modules_alias: &'a Option<ModulesAlias>,
        slots: &'a Option<Slots>,
    ) -> Self {
        Self {
            iter,
            modules_alias,
            slots,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = super::Result<Device>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.iter.next()?.ok()?.into_path();
        Some(LinuxSysfs::read_device(
            path,
            self.modules_alias,
            self.slots,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::prelude::PermissionsExt;

    use super::*;
    use pretty_assertions::assert_eq;
    use pretty_assertions::assert_str_eq;
    use tempfile::tempdir;

    const DEV00_1F_3: [u8; 64] = [
        0x86, 0x80, 0x22, 0x8c, 0x03, 0x00, 0x80, 0x02, 0x05, 0x00, 0x05, 0x0c, 0x00, 0x00, 0x00,
        0x00, 0x04, 0x30, 0xe1, 0xa2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28,
        0x10, 0xe5, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0a, 0x03, 0x00, 0x00,
    ];

    const DEV06_00_0: [u8; 64] = [
        0x2b, 0x10, 0x34, 0x05, 0x07, 0x00, 0x90, 0x02, 0x00, 0x00, 0x00, 0x03, 0x10, 0x20, 0x00,
        0x00, 0x08, 0x00, 0x00, 0xa1, 0x00, 0x00, 0x80, 0xa2, 0x00, 0x00, 0x00, 0xa2, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28,
        0x10, 0xe5, 0x05, 0x00, 0x00, 0x00, 0x00, 0xdc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x0a, 0x01, 0x10, 0x20,
    ];

    #[test]
    fn init_no_dir() {
        let path = "/7ecc5f6b4aadb8e641a07d3cea6e8c6fa43050c916e69eac7e300c3b25172cb6";
        let result = LinuxSysfs::new(path).access().unwrap_err();
        assert_str_eq!(
            "/7ecc5f6b4aadb8e641a07d3cea6e8c6fa43050c916e69eac7e300c3b25172cb6: No such file or directory (os error 2)",
             result.to_string()
        );
    }

    #[test]
    fn init_empty_dir() {
        let dir = tempdir().unwrap();
        let result = LinuxSysfs::new(dir.path()).access();
        assert!(result.is_ok());
    }

    #[test]
    fn scan_valid_adresses() {
        let sample = vec!["0000:04:00.0", "0000:08:00.0"];
        let dir = tempdir().unwrap();
        let path = dir.path();
        for dev in &sample {
            fs::create_dir_all(path.join("devices").join(dev)).unwrap();
        }
        let access = LinuxSysfs::new(path).access().unwrap();
        let mut result: Vec<_> = access
            .scan()
            .map(|addr| addr.unwrap().to_string())
            .collect();
        result.sort();
        assert_eq!(sample, result);
    }

    #[test]
    fn scan_invalid_adresses() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let iter = 'm'..'z';
        for dev in iter.clone() {
            fs::create_dir_all(path.join("devices").join(dev.to_string())).unwrap();
        }
        let access = LinuxSysfs::new(path).access().unwrap();
        let result: Vec<_> = access.scan().map(|a| a.is_err()).collect();
        let sample: Vec<_> = iter.map(|_| true).collect();
        assert_eq!(sample, result);
    }

    #[test]
    fn valid_device() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let device = "0000:00:1f.3";
        let dev_path = path.join("devices").join(device);
        fs::create_dir_all(&dev_path).unwrap();
        fs::write(dev_path.join("config"), DEV00_1F_3).unwrap();

        let sample: Address = device.parse().unwrap();
        let access = LinuxSysfs::new(path).access().unwrap();
        let result = access.device(sample.clone()).unwrap();

        assert_eq!(sample, result.address);
    }

    #[test]
    fn invalid_device() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let device = "0000:00:1f.3";
        let dev_path = path.join("devices").join(device);
        fs::create_dir_all(&dev_path).unwrap();
        fs::write(dev_path.join("config"), b"invalid").unwrap();

        let sample: Address = device.parse().unwrap();
        let access = LinuxSysfs::new(path).access().unwrap();
        let result = access.device(sample).unwrap_err();

        assert_eq!(
            "unable to parse configuration space data".to_string(),
            result.to_string()
        );
    }

    #[test]
    fn valid_iter() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let dev_dir = path.join("devices").join("0000:00:1f.3");
        fs::create_dir_all(&dev_dir).unwrap();
        fs::write(dev_dir.join("config"), DEV00_1F_3).unwrap();

        let dev_dir = path.join("devices").join("0000:06:00.0");
        fs::create_dir_all(&dev_dir).unwrap();
        fs::write(dev_dir.join("config"), DEV06_00_0).unwrap();

        let access = LinuxSysfs::new(path).access().unwrap();
        let mut result = access
            .iter()
            .map(|result| result.unwrap().address.to_string())
            .collect::<Vec<_>>();
        result.sort();

        assert_eq!(vec!["0000:00:1f.3", "0000:06:00.0"], result);
    }

    #[test]
    fn invalid_iter() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        let dev_dir = path.join("devices").join("0000:00:00.0");
        fs::create_dir_all(&dev_dir).unwrap();
        let conf_path = dev_dir.join("config");
        fs::write(&conf_path, DEV00_1F_3).unwrap();
        let conf_file = fs::File::open(conf_path).unwrap();
        let mut perms = conf_file.metadata().unwrap().permissions();
        perms.set_mode(0o000);
        conf_file.set_permissions(perms).unwrap();

        let dev_dir = path.join("devices").join("0000:06:00.0");
        fs::create_dir_all(&dev_dir).unwrap();
        fs::write(dev_dir.join("config"), DEV06_00_0).unwrap();

        let access = LinuxSysfs::new(path).access().unwrap();
        let result = access.iter().collect::<Result<Vec<_>, AccessError>>();

        assert!(matches!(
            result,
            Err(AccessError::File {
                source: io::Error { .. },
                ..
            })
        ));
    }
}
