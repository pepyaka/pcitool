//use std::collections::HashMap;
use std::iter::FromIterator;
use std::convert::TryFrom;
use std::num::ParseIntError;

use regex::Regex;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::device::{Address, Header, HeaderError, ParseAddressError, Device};
use crate::access::{AccessMethod};

#[derive(Debug)]
pub struct Dump {
    data: Vec<Device>,
    errors: Vec<(usize, ParseDumpError)>,
}

#[derive(Debug, Error)]
pub enum ParseDumpError {
    #[error("address parsing error: {0}")]
    ParseAddress(#[source] ParseAddressError),
    #[error("offset parsing error: {0}")]
    ParseOffset(#[source] ParseIntError),
    #[error("offset should be less than 0x1000")]
    OffsetOverflow,
    #[error("octet #{0} parsing error: {1}")]
    ParseOctet(usize, #[source] ParseIntError),
    #[error("failed octet #{0}")]
    OctetFailed(usize),
    #[error("header parsing error: {0}")]
    Header(#[source] HeaderError),
}

//impl Dump {
//    pub fn lines<'a, T: Into<PathBuf>>(path: T) -> io::Result<std::str::Lines<'a>> {
//        fs::read_to_string(path.into())
//            .map(|file| file.lines())
//    }
//}

impl AccessMethod for Dump {
    type Iter = std::vec::IntoIter<Device>;
    //fn init() -> Result<Self, AccessError> {
    //}
    fn iter(&self) -> Self::Iter {
        self.data.clone().into_iter()
    }
}

impl<'a> FromIterator<&'a str> for Dump {
    fn from_iter<I: IntoIterator<Item=&'a str>>(lines: I) -> Self {
        lazy_static! {
            static ref ADDRESS_RE: Regex =
                Regex::new(r"^((0?[[:xdigit:]]{4})?:?[[:xdigit:]]{2}:[[:xdigit:]]{2}\.[[:xdigit:]])\s")
                    .unwrap();
            static ref DATA_RE: Regex =
                Regex::new(r"^([[:xdigit:]]{2,3}):((\s[[:xdigit:]]{2}){16})")
                    .unwrap();
        }

        let mut data = Vec::new();
        let mut errors = Vec::new();

        let mut cur_addr = None;
        let mut cur_data = [0u8; 4096]; 
        for (n, line) in lines.into_iter().enumerate() {
            if let Some(addr) = ADDRESS_RE.captures(line)
                .and_then(|c| c.get(1).map(|s| s.as_str()))
            {
                match addr.parse::<Address>() {
                    Ok(addr) => {
                        if let (Some(address), Ok(header)) = 
                            (cur_addr, Header::try_from(&cur_data[..64]))
                        {
                            data.push(Device::new(address, header));
                        }
                        cur_addr = Some(addr);
                    },
                    Err(err) => {
                        cur_addr = None;
                        errors.push((n, ParseDumpError::ParseAddress(err)));
                    }
                }
                cur_data.fill(0);
            }
            if let Some((Some(offset), Some(sixteen))) = DATA_RE.captures(line)
                .map(|c| (c.get(1).map(|s| s.as_str()), c.get(2).map(|s| s.as_str())))
            {
                match usize::from_str_radix(offset, 16) {
                    Ok(offset) => {
                        if offset < 4096 {
                            let mut octets = sixteen.trim_start().split_whitespace();
                            for i in 0..16 {
                                if let Some(octet) = octets.next() {
                                    match u8::from_str_radix(octet, 16) {
                                        Ok(byte) => {
                                            cur_data[offset + i] = byte;
                                        },
                                        Err(err) => {
                                            errors.push((n, ParseDumpError::ParseOctet(i, err)));
                                        },
                                    }
                                } else {
                                    errors.push((n, ParseDumpError::OctetFailed(i)));
                                }
                            }
                        } else {
                            errors.push((n, ParseDumpError::OffsetOverflow));
                        }
                    },
                    Err(err) => {
                        errors.push((n, ParseDumpError::ParseOffset(err)));
                    },
                }

            }
        }
        if let (Some(address), Ok(header)) = (cur_addr, Header::try_from(&cur_data[..64])) {
            data.push(Device::new(address, header));
        }
        Dump { data, errors }
    }
}
impl Iterator for Dump {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}





#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    use crate::device::HeaderCommon;

    #[test]
    fn first() {
        let f = fs::read_to_string("./tests/data/lspci.dump")
            .expect("cannot read file");
        let dump: Dump = f.lines()
            .collect();
        for d in dump.iter() {
            let HeaderCommon { vendor_id, device_id, .. } = d.header.layout.common();
            println!("{:#}: {:X}:{:X}", d.address, vendor_id, device_id);
        }
        for e in dump.errors {
            println!("{:?}", e);
        }
    }
}
