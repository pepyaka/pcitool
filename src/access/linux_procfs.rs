//!  The /sys filesystem on Linux 2.6 and newer. The standard header of the config space is
//!  available to all users, the  rest  only  to  root.  Supports  extended configuration space,
//!  PCI domains, VPD (from Linux 2.6.26), physical slots (also since Linux 2.6.26) and information
//!  on attached kernel drivers.

use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::PathBuf,
    str::FromStr,
};

use thiserror::Error;
use walkdir::WalkDir;

use super::{AccessError, AccessMethod};
use crate::device::{Address, ConfigurationSpace, Device, Resource, ResourceEntry};

#[derive(Debug, Error)]
pub enum LinuxProcfsError {
    #[error("{path} read problem")]
    ReadDir { path: PathBuf, source: io::Error },
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("bus/devfn, vendor/device, irq, base_addr x 6 are mandatory fields")]
    InfoListMandatoryField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxProcfs {
    path: PathBuf,
    info: InfoEntries,
}

type InfoEntries = HashMap<Address, InfoEntry>;

impl LinuxProcfs {
    pub const PATH: &'static str = "/proc/bus/pci";
    pub fn init(path: impl Into<PathBuf>) -> super::Result<Self> {
        let path = path.into();
        let is_dir = fs::metadata(&path)
            .map_err(|source| AccessError::File {
                path: path.clone(),
                source,
            })?
            .is_dir();
        if !is_dir {
            return Err(AccessError::File {
                path,
                source: io::Error::new(io::ErrorKind::Other, "is not a directory"),
            });
        }
        // Devices information /proc/bus/pci/devices
        let info_path = path.join("devices");
        let f = fs::File::open(&info_path).map_err(|source| AccessError::File {
            path: info_path,
            source,
        })?;
        let reader = BufReader::new(f);
        let info = reader
            .lines()
            .filter_map(|line| {
                let line = line.ok()?;
                let entry: InfoEntry = line.parse().ok()?;
                Some((entry.address(), entry))
            })
            .collect();
        Ok(Self { path, info })
    }
    fn address_from_path(path: impl Into<PathBuf>) -> super::Result<Address> {
        let path = path.into();
        let cpath = path.clone();
        let mut components = cpath.iter().rev().filter_map(|s| s.to_str());
        let devfn = components.next().ok_or(AccessError::File {
            path: path.clone(),
            source: io::Error::new(io::ErrorKind::Other, "unreadable devfn path component"),
        })?;
        let bus = components.next().ok_or(AccessError::File {
            path,
            source: io::Error::new(io::ErrorKind::Other, "unreadable bus path component"),
        })?;
        let address = format!("{}:{}", bus, devfn);
        address
            .parse()
            .map_err(|source| AccessError::ParseAddress { address, source })
    }
    // Config spaces /proc/bus/pci/xx/xx.x iterator
    fn device_entries(&self) -> walkdir::IntoIter {
        WalkDir::new(&self.path)
            .min_depth(2)
            .max_depth(2)
            .follow_links(true)
            .into_iter()
    }
    fn read_device(path: impl Into<PathBuf>, info: &InfoEntries) -> super::Result<Device> {
        let path = path.into();
        let address = Self::address_from_path(&path)?;
        let bytes = fs::read(&path).map_err(|source| AccessError::File { path, source })?;
        let mut device = bytes
            .as_slice()
            .try_into()
            .map(|cs: ConfigurationSpace| Device::new(address.clone(), cs))
            .map_err(|_| AccessError::ConfigurationSpace)?;
        if let Some(&InfoEntry {
            irq,
            base_addr,
            rom_addr,
            base_size,
            rom_size,
            ..
        }) = info.get(&address)
        {
            device.irq = Some(irq);
            let mut entries = [ResourceEntry::default(); 6];
            let base_size = base_size.unwrap_or_default();
            for (i, entry) in entries.iter_mut().enumerate() {
                let start = base_addr[i];
                let end = (start + base_size[i]).saturating_sub(1);
                *entry = ResourceEntry {
                    start,
                    end,
                    flags: 0,
                };
            }
            device.resource = Some(Resource {
                entries,
                rom_entry: {
                    let start = rom_addr.unwrap_or(0);
                    ResourceEntry {
                        start,
                        end: (start + rom_size.unwrap_or(0)).saturating_sub(1),
                        flags: 0,
                    }
                },
            });
        }
        Ok(device)
    }
}

impl<'a> AccessMethod<'a> for LinuxProcfs {
    type Scan = Scan;
    type Iter = Iter<'a>;
    fn device(&self, address: Address) -> super::Result<Device> {
        let path = self
            .path
            .join(format!("{:02x}", address.bus))
            .join(format!("{:02x}.{}", address.device, address.function));
        if path.is_file() {
            Self::read_device(path, &self.info)
        } else {
            // Paths with domains (ex: /proc/bus/pci/0001:02/)
            let path = self
                .path
                .join(format!("{:04x}:{:02x}", address.domain, address.bus))
                .join(format!("{:02x}.{}", address.device, address.function));
            Self::read_device(path, &self.info)
        }
    }
    fn scan(&'a self) -> Self::Scan {
        Scan::new(self.device_entries())
    }
    fn iter(&'a self) -> Self::Iter {
        Iter::new(self.device_entries(), &self.info)
    }
}

#[derive(Error, Debug)]
pub enum ProcfsDeviceError {
    #[error("parse address fail from domain/bus: {bus}, dev/fn: {devfn}")]
    Address { bus: String, devfn: String },
    #[error("bus/dev/fn file: {path} read problem")]
    BusDevFnPath { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcfsDevice {
    address: Address,
    path: PathBuf,
}

#[derive(Error, Debug)]
pub enum InfoEntryError {
    #[error("mandatory fields: bus, devfn, vendor, device, irq, base_address x 6")]
    MandatoryField,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InfoEntry {
    bus_number: u8,
    devfn: u8,
    vendor: u16,
    device: u16,
    irq: usize,
    base_addr: [u64; 6],
    rom_addr: Option<u64>,
    base_size: Option<[u64; 6]>,
    rom_size: Option<u64>,
    drv_name: Option<String>,
}

impl InfoEntry {
    pub fn address(&self) -> Address {
        let &Self {
            bus_number, devfn, ..
        } = self;
        Address {
            domain: 0,
            bus: bus_number,
            device: (devfn >> 3) & 0x1f,
            function: devfn & 0x07,
        }
    }
}

impl FromStr for InfoEntry {
    type Err = InfoEntryError;

    #[cfg(not(feature = "lib_proc_baseaddr_parse"))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let (bus_number, devfn) = fields
            .next()
            .ok_or(InfoEntryError::MandatoryField)?
            .split_at(2);
        let bus_number = u8::from_str_radix(bus_number, 16)?;
        let devfn = u8::from_str_radix(devfn, 16)?;
        let (vendor, device) = fields
            .next()
            .ok_or(InfoEntryError::MandatoryField)?
            .split_at(4);
        let vendor = u16::from_str_radix(vendor, 16)?;
        let device = u16::from_str_radix(device, 16)?;
        let irq = fields.next().ok_or(InfoEntryError::MandatoryField)?;
        let irq = usize::from_str_radix(irq, 16)?;
        let base_addr = {
            let mut result = [0u64; 6];
            for ba in result.iter_mut() {
                *ba = fields
                    .next()
                    .and_then(|s| u64::from_str_radix(s, 16).ok())
                    .ok_or(InfoEntryError::MandatoryField)?;
            }
            result
        };
        let rom_addr = fields.next().and_then(|s| u64::from_str_radix(s, 16).ok());
        let base_size = || -> Option<[u64; 6]> {
            let mut result = [0u64; 6];
            for bs in result.iter_mut() {
                let s = fields.next()?;
                *bs = u64::from_str_radix(s, 16).ok()?;
            }
            Some(result)
        }();
        let rom_size = fields.next().and_then(|s| u64::from_str_radix(s, 16).ok());
        let drv_name = fields.next().map(Into::into);
        Ok(Self {
            bus_number,
            devfn,
            vendor,
            device,
            irq,
            base_addr,
            rom_addr,
            base_size,
            rom_size,
            drv_name,
        })
    }

    #[cfg(feature = "lib_proc_baseaddr_parse")]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use std::ffi::{c_int, c_long, CString};
        let s = CString::new(s).unwrap();
        let format = "%x %x %x %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx %08lx";
        let format = CString::new(format).unwrap();
        let mut dfn = 0 as c_int;
        let mut vend = 0 as c_int;
        let mut irq = 0 as c_int;
        let mut base_addr = [0 as c_long; 6];
        let mut rom_base_addr = 0 as c_long;
        let mut size = [0 as c_long; 6];
        let mut rom_size = 0 as c_long;
        let base_addr_ptr = base_addr.as_mut_ptr();
        let size_ptr = size.as_mut_ptr();
        let _cnt = unsafe {
            libc::sscanf(
                s.as_ptr(),
                format.as_ptr(),
                &mut dfn,
                &mut vend,
                &mut irq,
                base_addr_ptr.offset(0),
                base_addr_ptr.offset(1),
                base_addr_ptr.offset(2),
                base_addr_ptr.offset(3),
                base_addr_ptr.offset(4),
                base_addr_ptr.offset(5),
                &mut rom_base_addr,
                size_ptr.offset(0),
                size_ptr.offset(1),
                size_ptr.offset(2),
                size_ptr.offset(3),
                size_ptr.offset(4),
                size_ptr.offset(5),
                &mut rom_size,
            )
        };
        Ok(Self {
            bus_number: (dfn >> 8) as u8,
            devfn: (dfn & 0xff) as u8,
            vendor: (vend >> 16) as u16,
            device: (vend & 0xffff) as u16,
            irq: irq as usize,
            base_addr: base_addr.map(|v| v as u64),
            rom_addr: Some(rom_base_addr as u64),
            base_size: Some(size.map(|v| v as u64)),
            rom_size: Some(rom_size as u64),
            drv_name: Default::default(),
        })
    }
}

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
        let path = self.iter.next()?.ok()?.into_path();
        Some(LinuxProcfs::address_from_path(path))
    }
}

pub struct Iter<'a> {
    iter: walkdir::IntoIter,
    info: &'a HashMap<Address, InfoEntry>,
}

impl<'a> Iter<'a> {
    pub fn new(iter: walkdir::IntoIter, info: &'a HashMap<Address, InfoEntry>) -> Self {
        Self { iter, info }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = super::Result<Device>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.iter.next()?.ok()?.into_path();
        Some(LinuxProcfs::read_device(path, self.info))
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

    #[cfg(not(feature = "lib_proc_baseaddr_parse"))]
    #[test]
    fn parse_info_list_line() {
        let data = "0200 10de1d12 91 b3000000 a000000c 0 b000000c 0 3001 b4000000 1000000 10000000 0 2000000 0 80 80000 nouveau";
        let result: InfoEntry = data.parse().unwrap();
        let sample = InfoEntry {
            bus_number: 0x02,
            devfn: 0x00,
            vendor: 0x10de,
            device: 0x1d12,
            irq: 0x91,
            base_addr: [0xb3000000, 0xa000000c, 0, 0xb000000c, 0, 0x3001],
            rom_addr: Some(0xb4000000),
            base_size: Some([0x1000000, 0x10000000, 0, 0x2000000, 0, 0x80]),
            rom_size: Some(0x80000),
            drv_name: Some("nouveau".into()),
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn init_no_dir() {
        let path = "/7ecc5f6b4aadb8e641a07d3cea6e8c6fa43050c916e69eac7e300c3b25172cb6";
        let result = LinuxProcfs::init(path).unwrap_err();
        assert_str_eq!(
            "/7ecc5f6b4aadb8e641a07d3cea6e8c6fa43050c916e69eac7e300c3b25172cb6: No such file or directory (os error 2)",
             result.to_string()
        );
    }

    #[test]
    fn init_empty_dir() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();
        let result = LinuxProcfs::init(path);
        assert!(result.is_ok());
    }

    #[test]
    fn scan_valid_device_paths() {
        let data = [("04", "00.0"), ("08", "00.0")];
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();
        for dev in &data {
            let bus_path = path.join(dev.0);
            fs::create_dir(&bus_path).unwrap();
            fs::write(bus_path.join(dev.1), "").unwrap();
        }
        let access = LinuxProcfs::init(path).unwrap();
        let mut result: Vec<_> = access
            .scan()
            .map(|addr| format!("{:#}", addr.unwrap()))
            .collect();
        result.sort();
        let sample: Vec<_> = data
            .iter()
            .map(|(bus, devfn)| format!("{}:{}", bus, devfn))
            .collect();
        assert_eq!(sample, result);
    }

    #[test]
    fn scan_invalid_device_paths() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();
        let iter = 'm'..'z';
        for dev in iter.clone() {
            let bus_path = path.join(dev.to_string());
            fs::create_dir(&bus_path).unwrap();
            fs::write(bus_path.join("invalid"), "").unwrap();
        }
        let access = LinuxProcfs::init(path).unwrap();
        let result: Vec<_> = access.scan().map(|a| a.is_err()).collect();
        let sample: Vec<_> = iter.map(|_| true).collect();
        assert_eq!(sample, result);
    }

    #[test]
    fn valid_device() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();
        let (bus, devfn) = ("00", "1f.3");
        let sample: Address = format!("{}:{}", bus, devfn).parse().unwrap();
        let bus_path = path.join(bus);
        fs::create_dir(&bus_path).unwrap();
        fs::write(bus_path.join(devfn), DEV00_1F_3).unwrap();
        let access = LinuxProcfs::init(path).unwrap();
        let result = access.device(sample.clone()).unwrap();
        assert_eq!(sample, result.address);
    }

    #[test]
    fn invalid_device() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();
        let (bus, devfn) = ("00", "1f.3");
        let address: Address = format!("{}:{}", bus, devfn).parse().unwrap();
        let bus_path = path.join(bus);
        fs::create_dir(&bus_path).unwrap();
        fs::write(bus_path.join(devfn), "invalid").unwrap();
        let access = LinuxProcfs::init(path).unwrap();
        let result = access.device(address).unwrap_err();
        assert_str_eq!(
            "unable to parse configuration space data".to_string(),
            result.to_string()
        );
    }

    #[test]
    fn valid_iter() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        fs::write(path.join("devices"), "").unwrap();

        let bus_dir = path.join("00");
        fs::create_dir(&bus_dir).unwrap();
        fs::write(bus_dir.join("1f.3"), DEV00_1F_3).unwrap();

        let bus_dir = path.join("06");
        fs::create_dir(&bus_dir).unwrap();
        fs::write(bus_dir.join("00.0"), DEV06_00_0).unwrap();

        let access = LinuxProcfs::init(path).unwrap();
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
        fs::write(path.join("devices"), "").unwrap();

        let bus_dir = path.join("00");
        fs::create_dir(&bus_dir).unwrap();
        let conf_path = bus_dir.join("1f.3");
        fs::write(&conf_path, DEV00_1F_3).unwrap();
        let conf_file = fs::File::open(conf_path).unwrap();
        let mut perms = conf_file.metadata().unwrap().permissions();
        perms.set_mode(0o000);
        conf_file.set_permissions(perms).unwrap();

        let bus_dir = path.join("06");
        fs::create_dir(&bus_dir).unwrap();
        fs::write(bus_dir.join("00.0"), DEV06_00_0).unwrap();

        let access = LinuxProcfs::init(path).unwrap();
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
