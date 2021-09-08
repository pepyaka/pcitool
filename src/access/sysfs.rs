//!  The /sys filesystem on Linux 2.6 and newer. The standard header of the config space is
//!  available to all users, the  rest  only  to  root.  Supports  extended configuration space,
//!  PCI domains, VPD (from Linux 2.6.26), physical slots (also since Linux 2.6.26) and information
//!  on attached kernel drivers.

use std::{
    fs,
    io,
    io::prelude::*,
    path::{ PathBuf, },
};
//use core::iter;

use core::convert::{TryFrom};
//use anyhow::{anyhow};

use crate::device::{Device, Header, Address};
use super::{AccessMethod, AccessError };

pub const SYSFS_PATH: &'static str = "/sys/bus/pci/devices";

pub struct SysFs{
    path: PathBuf,
}
impl SysFs {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { path: path.into() }
    }
}

impl AccessMethod for SysFs {
    type Iter = Iter;
    fn read(&self, address: Address) -> Result<Device, AccessError> {
        let mut buf = [0u8; 4096];
        let path = self.path.join(address.to_string()).join("config");
        let mut f = fs::File::open(&path)?;
        let length = f.read(&mut buf)?;
        let header = Header::try_from(&buf[..length])?;
        let mut device = Device::new(address, header);
        device.label = fs::read_to_string(&path.join("label")).ok();
        Ok(device)
    }
    fn iter(&self) -> Self::Iter {
        Iter { buf: [0; 4096], entries: fs::read_dir(&self.path) }
    }
}

pub struct Iter {
    buf: [u8; 4096],
    entries: io::Result<fs::ReadDir>,
}

impl Iterator for Iter {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.entries.as_mut().ok()?
            .next()?.ok()
            .map(|de| de.path())?;
        let mut config = fs::File::open(path.join("config")).ok()?;
        let length = config.read(&mut self.buf).ok()?;
        let address = path.file_name()?.to_str()?.parse().ok()?;
        let header = Header::try_from(&self.buf[..length]).ok()?;
        let mut device = Device::new(address, header);
        device.label = fs::read_to_string(&path.join("label")).ok();
        Some(device)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn iter() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/vfs")
            .join(SYSFS_PATH.strip_prefix("/").unwrap());
        let mut devices = SysFs::new(path).iter()
            .collect::<Vec<_>>();
        devices.sort();
        let result = devices.iter()
            .map(|d| format!("{:#}", d.address))
            .collect::<Vec<_>>();
        assert_eq!(vec!["00:00.0","00:1f.3","03:00.0"], result);
    }
    #[test]
    fn read() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/vfs")
            .join(SYSFS_PATH.strip_prefix("/").unwrap());
        let result = SysFs::new(path)
            .read("03:00.0".parse().unwrap())
            .map(|d| { 
                let c = d.header.layout.common();
                (c.vendor_id, c.device_id) 
            })
            .unwrap();
        assert_eq!((0x10EC, 0x522A), result);
    }
}
