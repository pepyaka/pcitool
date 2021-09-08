use std::path::Path;
use std::fs;
use std::io::{self, prelude::*, SeekFrom, };
use std::convert::{TryInto};

use super::{AccessMethod, AccessError};
use crate::device::{Address, Device};

pub const DEV_PORT_PATH: &'static str = "/dev/port";
const CONFIG_ADDRESS: u32 = 0xCF8;


#[derive(Debug)]
pub struct DevPort<'a> {
    path: &'a Path,
    address: Address,
    offset: u8,
}
impl<'a> DevPort<'a> {
    pub fn new(path: &'a Path, address: Address) -> Self {
        Self { path, address, offset: 0 }
    }
    /// The CONFIG_ADDRESS is a 32-bit register with the format shown in following figure. Bit 31
    /// is an enable flag for determining when accesses to CONFIG_DATA should be translated to
    /// configuration cycles. Bits 23 through 16 allow the configuration software to choose a
    /// specific PCI bus in the system. Bits 15 through 11 select the specific device on the PCI
    /// Bus. Bits 10 through 8 choose a specific function in a device (if the device supports
    /// multiple functions).
    ///
    /// The least significant byte selects the offset into the 256-byte configuration space
    /// available through this method. Since all reads and writes must be both 32-bits and aligned
    /// to work on all implementations, the two lowest bits of CONFIG_ADDRESS must always be zero,
    /// with the remaining six bits allowing you to choose each of the 64 32-bit words. If you
    /// don't need all 32 bits, you'll have to perform the unaligned access in software by aligning
    /// the address, followed by masking and shifting the answer.
    ///
    /// | 31 | 30 - 24 | 23 - 16 | 15 - 11 | 10 - 8 | 7 - 0 |
    /// |-|-|-|-|-|-|
    /// | Enable Bit | Reserved | Bus Number | Device Number | Function Number | Register Offset |
    fn config_address(&self) -> u32 {
        let Address { bus, device, function, .. } = self.address;
        let (bus, device, function, offset) =
            (bus as u32, device as u32, function as u32, self.offset as u32);
        (bus << 16) | (device << 11) | (function << 8) | (offset & 0xFC) | 0x80000000
    }
}

//impl AccessMethod for DevPort {
//    fn init() -> Result<Self, AccessError> {
//        let file = fs::File::open(DEV_PORT_PATH)
//            .map_err(AccessError::Io)?;
//        Ok(Self { file })
//    }
//    fn device(address: Address) -> Result<Address, AccessError> {
//        Err(AccessError::Platform)
//    }
//}

impl<'a> Read for DevPort<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.path)?;

        let init_offset = self.offset as usize;
        let end = self.offset.saturating_add((buf.len()).min(u8::MAX as usize) as u8);

        while self.offset < end {

            if file.seek(SeekFrom::Start(CONFIG_ADDRESS.into())).is_err() {
                break;
            }
            let data_address = self.config_address().to_le_bytes();
            if !matches!(file.write(&data_address), Ok(4)) {
                break;
            }
            if file.seek(SeekFrom::Current((self.offset % 4).into())).is_err() {
                break;
            }
            let buf_start = self.offset as usize - init_offset;
            let buf_end = buf.len().min(buf_start + 4 - (self.offset % 4) as usize);
            if let Ok(bytes_read @ 1..=4) = file.read(&mut buf[buf_start..buf_end]) {
                self.offset = self.offset.saturating_add(bytes_read as u8);
            } else {
                break;
            }
        }
        Ok(self.offset as usize - init_offset)
    }
}

impl<'a> Seek for DevPort<'a> {
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let offset = match style {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::End(n) => n + u8::MAX as i64,
            SeekFrom::Current(n) => n + self.offset as i64,
        };
        self.offset = offset.try_into()
            .map_err(|_| io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid seek to position: {}", offset)
            ))?;
        Ok(self.offset as u64)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn config_address() {
        let addr = Address { domain: 0, bus: 1, device: 2, function: 3 };
        let dev_port = DevPort::new(Path::new("/dev/null"), addr);
        let address = dev_port.config_address(); 
        assert_eq!(0x80011300u32, address);
    }

    #[test]
    fn read() {
        let temp_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&temp_path).unwrap();
        file.write_all(&[0u8; 0x1000]).unwrap();
        file.seek(SeekFrom::Start((CONFIG_ADDRESS + 4).into())).unwrap();
        file.write_all(&[0x00,0x55,0xAA,0xFF]).unwrap();
        file.sync_all().unwrap();

        let addr = Address { domain: 0, bus: 0, device: 0, function: 0 };
        let mut dev_port = DevPort::new(&temp_path, addr);

        let mut one = [0u8; 1];
        dev_port.read_exact(&mut one).unwrap(); // Cursor: 0
        assert_eq!([0x00], one, "One byte");
        
        let mut two = [0u8; 2];
        dev_port.read_exact(&mut two).unwrap(); // Cursor: 1
        assert_eq!([0x55,0xAA], two, "Two bytes");

        let mut three = [0u8; 3];
        dev_port.read_exact(&mut three).unwrap(); // Cursor: 3
        assert_eq!([0xFF,0x00,0x55], three, "Three bytes");

        let fill = std::iter::repeat([0xAA,0xFF,0x00,0x55,])
            .flatten()
            .take(250)
            .collect::<Vec<_>>();
        let mut buf = vec![0u8; 250];
        dev_port.read(&mut buf).unwrap(); // Cursor: 6
        assert_eq!(fill, buf, "Fill whole buf");

        let mut overflow = [0u8; 4];
        dev_port.read(&mut overflow).unwrap();
        assert_eq!([0u8; 4], overflow, "Overflow");
    }

    #[test]
    fn seek() {
        let temp_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path).unwrap();
        file.write_all(&[0u8; 0x1000]).unwrap();
        file.seek(SeekFrom::Start((CONFIG_ADDRESS + 4).into())).unwrap();
        file.write_all(&[0x00,0x55,0xAA,0xFF]).unwrap();
        file.sync_all().unwrap();
        
        let addr = Address { domain: 0, bus: 0, device: 0, function: 0 };
        let mut dev_port = DevPort::new(&temp_path, addr);

        let x = rand::random::<u8>();
        let mut start = [0];
        let mut end = [0];
        dev_port.seek(SeekFrom::Start(x as u64)).unwrap();
        dev_port.read(&mut start).unwrap();
        dev_port.seek(SeekFrom::End(x as i64 - 255)).unwrap();
        dev_port.read(&mut end).unwrap();
        assert_eq!(start, end, "Start: {}, End: {}", start[0], end[0]);

        let x = rand::random::<f32>().clamp(4.0, 251.0) as u64;
        let mut fwd = [0];
        let mut bwd = [0];
        dev_port.seek(SeekFrom::Start(x)).unwrap();
        dev_port.seek(SeekFrom::Current(4)).unwrap();
        dev_port.read(&mut fwd).unwrap();
        dev_port.seek(SeekFrom::Current(-9)).unwrap();
        dev_port.read(&mut bwd).unwrap();
        assert_eq!(fwd, bwd, "Forward: {}, Backward: {}", fwd[0], bwd[0]);
    }
}
