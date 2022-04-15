use std::io;


use thiserror::Error;
#[cfg(feature = "clap")]
use clap::ArgEnum;

use crate::device::{Device, Address, };

//pub mod dev_port;
// pub mod sysfs;
pub mod dump;



pub trait AccessMethod<'a>: core::fmt::Debug
where
    Self: Sized
{
    type Iter: Iterator<Item = Device>;

    //fn init(&self) -> Result<Self, AccessError>;
    fn read(&'a self, addr: Address) -> Result<Device, AccessError> {
        self.iter()
            .find(|d| d.address == addr)
            .map(|d| d.clone())
            .ok_or(AccessError::NoAddress(addr))
    }
    fn iter(&'a self) -> Self::Iter;
    //fn device(address: Address) -> Result<Address, AccessError>;
    //fn control(&mut self, command: Command) -> Result<Command, AccessError>;
    //fn status(&mut self, reset: Status) -> Result<Status, AccessError>;
    //fn devices() -> Devices;
}

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("No addressed device {0}")]
    NoAddress(Address),
    #[error("This method unavailable on this platform")]
    Platform,
    #[error("I/O problem {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "clap", derive(ArgEnum))]
pub enum PreferredMethod {
    LinuxSysfs,
    LinuxProc,
    IntelConf1,
    IntelConf2,
    #[cfg(target_os = "freebsd")]
    FbsdDevice,
    #[cfg(target_os = "netbsd")]
    NbsdDevice,
    #[cfg(target_os = "openbsd")]
    ObsdDevice,
    #[cfg(target_os = "macos")]
    Darwin,
    Dump,
}

// #[derive(Debug)]
// pub enum UsedMethod {
//     LinuxSysfs,
//     LinuxProc,
//     IntelConf1,
//     IntelConf2,
//     #[cfg(target_os = "freebsd")]
//     FbsdDevice,
//     #[cfg(target_os = "netbsd")]
//     NbsdDevice,
//     #[cfg(target_os = "openbsd")]
//     ObsdDevice,
//     #[cfg(target_os = "macos")]
//     Darwin,
//     Dump(Dump),
//     Unavailable,
// }
