use std::io;

use thiserror::Error;

use crate::device::{Device, Address, HeaderTypeError, Command, Status};

//pub mod dev_port;
pub mod sysfs;
pub mod dump;


pub trait AccessMethod
where
    Self: Sized
{
    type Iter: Iterator<Item = Device>;

    //fn init(&self) -> Result<Self, AccessError>;
    fn read(&self, addr: Address) -> Result<Device, AccessError> {
        self.iter()
            .find(|d| d.address == addr)
            .map(|d| d.clone())
            .ok_or(AccessError::NoAddress(addr))
    }
    fn iter(&self) -> Self::Iter;
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
    #[error("device header {0}")]
    DeviceHeaderType(#[from] HeaderTypeError),
}

//pub struct Access;
//impl Access {
//    pub fn new<T: AccessMethod>(method: &str) -> impl Iterator<Item = Device<T>> {
//        match method {
//            "linux-sysfs" => sysfs::SysFs::new(),
//            _ => todo!(),
//        }
//    }
//}
