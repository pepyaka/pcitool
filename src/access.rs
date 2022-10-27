use std::{io, iter, path::PathBuf, collections::HashMap};

use thiserror::Error;

use crate::device::{address::ParseAddressError, Address, Device};

use dump::{Dump, DumpError};
use linux_procfs::LinuxProcfs;
use linux_sysfs::LinuxSysfs;

//pub mod dev_port;
pub mod dump;
pub mod linux_procfs;
pub mod linux_sysfs;

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("No addressed device {0}")]
    NoAddress(Address),
    #[error("address '{address}' parse problem: {source}")]
    ParseAddress {
        address: String,
        source: ParseAddressError,
    },
    #[error("This method unavailable on this platform")]
    Platform,
    #[error("{path}: {source}")]
    File { path: PathBuf, source: io::Error },
    #[error("unable to parse configuration space data")]
    ConfigurationSpace,
    #[error(transparent)]
    Dump(#[from] DumpError),
    #[error("linux-sysfs access {0}")]
    LinuxSysfs(#[from] linux_sysfs::LinuxSysfsError),
}

impl PartialEq for AccessError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::File {
                    path: self_path,
                    source: self_source,
                },
                Self::File {
                    path: other_path,
                    source: other_source,
                },
            ) => self_path.eq(other_path) && self_source.kind().eq(&other_source.kind()),
            (s, o) => s.eq(o),
        }
    }
}

impl Eq for AccessError {}

pub type Result<T> = core::result::Result<T, AccessError>;
pub type Slots = HashMap<Address, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Access {
    Void(Void),
    Dump(Dump),
    LinuxSysfs(LinuxSysfs),
    LinuxProcfs(LinuxProcfs),
}

impl Access {
    pub fn init() -> Result<Self> {
        LinuxSysfs::default()
            .access()
            .or_else(|_| LinuxProcfs::init(LinuxProcfs::PATH).map(Into::into))
            .or_else(|_| Void::init().map(Into::into))
    }
    pub fn device(&self, addr: Address) -> Result<Device> {
        match self {
            Self::Void(a) => a.device(addr),
            Self::Dump(a) => a.device(addr),
            Self::LinuxSysfs(a) => a.device(addr),
            Self::LinuxProcfs(a) => a.device(addr),
        }
    }
    pub fn scan(&self) -> Box<dyn Iterator<Item = Result<Address>> + '_> {
        match self {
            Self::Void(a) => Box::new(a.scan()),
            Self::Dump(a) => Box::new(a.scan()),
            Self::LinuxSysfs(a) => Box::new(a.scan()),
            Self::LinuxProcfs(a) => Box::new(a.scan()),
        }
    }
    pub fn iter(&self) -> Box<dyn Iterator<Item = Result<Device>> + '_> {
        match self {
            Self::Void(a) => Box::new(a.iter()),
            Self::Dump(a) => Box::new(a.iter()),
            Self::LinuxSysfs(a) => Box::new(a.iter()),
            Self::LinuxProcfs(a) => Box::new(a.iter()),
        }
    }
    pub fn vital_product_data(&self, addr: Address) -> io::Result<Vec<u8>> {
        match self {
            Self::Void(a) => a.vital_product_data(addr),
            Self::Dump(a) => a.vital_product_data(addr),
            Self::LinuxSysfs(a) => a.vital_product_data(addr),
            Self::LinuxProcfs(a) => a.vital_product_data(addr),
        }
    }
}

impl Default for Access {
    fn default() -> Self {
        Self::Void(Void)
    }
}

impl From<Void> for Access {
    fn from(a: Void) -> Self {
        Self::Void(a)
    }
}

impl From<Dump> for Access {
    fn from(a: Dump) -> Self {
        Self::Dump(a)
    }
}

impl From<LinuxSysfs> for Access {
    fn from(a: LinuxSysfs) -> Self {
        Self::LinuxSysfs(a)
    }
}

impl From<LinuxProcfs> for Access {
    fn from(a: LinuxProcfs) -> Self {
        Self::LinuxProcfs(a)
    }
}

pub trait AccessMethod<'a> {
    type Scan: Iterator<Item = Result<Address>>;
    type Iter: Iterator<Item = Result<Device>>;
    fn device(&'a self, addr: Address) -> Result<Device> {
        self.iter()
            .find_map(|result| {
                result
                    .ok()
                    .filter(|Device { address, .. }| address == &addr)
            })
            .ok_or(AccessError::NoAddress(addr))
    }
    fn scan(&'a self) -> Self::Scan;
    fn iter(&'a self) -> Self::Iter;
    fn vital_product_data(&'a self, _: Address) -> io::Result<Vec<u8>> {
        Err(io::ErrorKind::Other.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Void;

impl Void {
    pub fn init() -> Result<Self> {
        Ok(Self)
    }
}

impl<'a> AccessMethod<'a> for Void {
    type Scan = iter::Empty<Result<Address>>;
    type Iter = iter::Empty<Result<Device>>;
    fn scan(&'a self) -> Self::Scan {
        iter::empty()
    }
    fn iter(&'a self) -> Self::Iter {
        iter::empty()
    }
}
