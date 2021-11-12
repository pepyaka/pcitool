//use std::collections::HashMap;
use core::str::FromStr;
use std::num::ParseIntError;
use std::str::Lines;

use regex::Regex;
use lazy_static::lazy_static;
use thiserror::Error;
use byte::{
    ctx::*,
    BytesExt,
};

use crate::device::ConfigurationSpaceSize;
use crate::device::{Address, address::ParseAddressError, Device};
use crate::access::AccessMethod;

#[derive(Debug)]
pub struct Dump(String);

impl Dump {
    pub fn new(dump: String) -> Self { Self(dump) }
}

impl<'a> AccessMethod<'a> for Dump {
    type Iter = Iter<'a>;
    fn iter(&'a self) -> Self::Iter {
        Iter { lines: self.0.lines(), addr: None }
    }
}


// #[derive(Debug, Error)]
// pub enum ParseDumpError {
//     #[error("address parsing error: {0}")]
//     ParseAddress(#[source] ParseAddressError),
//     #[error("offset parsing error: {0}")]
//     ParseOffset(#[source] ParseIntError),
//     #[error("offset should be less than 0x1000")]
//     OffsetOverflow,
//     #[error("octet #{0} parsing error: {1}")]
//     ParseOctet(usize, #[source] ParseIntError),
//     #[error("failed octet #{0}")]
//     OctetFailed(usize),
//     // #[error("header parsing error: {0}")]
//     // Header(#[source] HeaderError),
// }

#[derive(Clone, Debug, PartialEq, Eq)]
struct DumpAddress(Address);

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum ParseDumpAddressError {
    #[error("address line regex mismatch")]
    LineRegex,
    #[error("can't parse address")]
    Address(#[from] ParseAddressError),
}

impl FromStr for DumpAddress {
    type Err = ParseDumpAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref ADDRESS_RE: Regex =
                Regex::new(r"^(([[:xdigit:]]{4})?:?[[:xdigit:]]{2}:[[:xdigit:]]{2}\.[[:xdigit:]])\s")
                .unwrap();
        }
        // First match as str
        let first_match = ADDRESS_RE.captures(s)
            .and_then(|caps| caps.get(1).map(|m| m.as_str()))
            .ok_or(ParseDumpAddressError::LineRegex)?;
        let address = Address::from_str(first_match)?;
        Ok(DumpAddress(address))
    }
}


// Hex dump line looks like:
// 20: 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15
#[derive(Clone, Debug, PartialEq, Eq)]
struct HexDumpLine {
    offset: usize,
    u8x16: [u8; 16],
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum ParseHexDumpLineError {
    #[error("hex dump line regex mismatch")]
    LineRegex,
    // Variants below should never happen, regex should not allow
    #[error("failed to parse hex value")]
    Hexadecimal(#[from] ParseIntError),
    #[error("offset {0} out of range")]
    OutOfRange(usize),
    #[error("octets count {0} < 16")]
    Short(usize),
}

impl FromStr for HexDumpLine {
    type Err = ParseHexDumpLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref DATA_RE: Regex =
                Regex::new(r"^([[:xdigit:]]{1,2}0):((\s[[:xdigit:]]{2}){16})")
                .unwrap();
        }
        let (offset, octets) = DATA_RE.captures(s)
            // Map first match to offset and second to 16 bytes string
            .and_then(|c| {
                let offset = c.get(1).map(|s| s.as_str())?;
                let octets = c.get(2).map(|s| s.as_str())?;
                Some((offset, octets))
            })
            .ok_or(ParseHexDumpLineError::LineRegex)?;
        let offset = usize::from_str_radix(offset, 16)?;
        if offset > 4095 {
            return Err(ParseHexDumpLineError::OutOfRange(offset));
        }
        let mut u8x16 = [0; 16];
        let mut octets = octets.trim_start().split_whitespace().take(16);
        for i in 0..16 {
            if let Some(octet) = octets.next() {
                let byte = u8::from_str_radix(octet, 16)?;
                u8x16[i] = byte;
            } else {
                return Err(ParseHexDumpLineError::Short(i));
            }
        }
        Ok(HexDumpLine { offset, u8x16 })
    }
}

pub struct Iter<'a> {
    lines: Lines<'a>,
    addr: Option<Address>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Device;
    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes = [0u8; 4096]; 
        let mut size = ConfigurationSpaceSize::from(0);
        while let Some(line) = self.lines.next() {
            if let Ok(DumpAddress(new_addr)) = line.parse() {
                if let Some(addr) = self.addr.replace(new_addr) {
                    let cs = bytes[..(size as usize)]
                        .read_with(&mut 0, LE).ok()?;
                    let device = Device::new(addr, cs);
                    return Some(device);
                }
            }
            if let Ok(HexDumpLine { offset, u8x16 }) = line.parse() {
                size = offset.into();
                bytes[offset .. offset + 16].copy_from_slice(&u8x16[..]);
            }
        }
        if let Some(addr) = self.addr.take() {
            let cs = bytes[..(size as usize)]
                .read_with(&mut 0, LE).ok()?;
            let device = Device::new(addr, cs);
            Some(device)
        } else {
            None
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_address_dump() {
        let DumpAddress(valid_result) = 
            "02:00.0 3D controller: NVIDIA Corporation GP108M [GeForce MX150] (rev a1)"
            .parse()
            .unwrap();
        let valid_sample: Address = "02:00.0".parse().unwrap();
        assert_eq!(valid_sample, valid_result, "Valid");

        let empty_result: Result<DumpAddress, ParseDumpAddressError> = "".parse();
        assert_eq!(Err(ParseDumpAddressError::LineRegex), empty_result, "Empty");
    }

    #[test]
    fn parse_hexdump_line() {
        let valid_result: HexDumpLine =
            "20: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f"
            .parse()
            .unwrap();
        let valid_sample = HexDumpLine {
            offset: 0x20,
            u8x16: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15],
        };
        assert_eq!(valid_sample, valid_result, "Valid");

        let regex_mismatch_result: Result<HexDumpLine, ParseHexDumpLineError> = "".parse();
        assert_eq!(Err(ParseHexDumpLineError::LineRegex), regex_mismatch_result, "Regex mismatch");
    }
    // #[test]
    // fn dump_struct_from_text() {
    //     let f = fs::read_to_string("./tests/data/lspci.dump")
    //         .expect("cannot read file");
    //     let dump: Dump = f.lines()
    //         .collect();
    //     let vec = dump.0;
    //     assert_eq!([0x86, 0x80], vec[0].1[0..2], "First bytes");
    //     assert_eq!([0xde, 0x10], vec[22].1[0..2], "0000:02:00.0 3D controller: NVIDIA Corporation GP108M [GeForce MX150]");
    //     assert_eq!([0xff, 0x01, 0x00, 0x00], vec[vec.len() - 1].1[0x3c..0x40], "Last bytes");
    // }

    #[test]
    fn iter_once_size_64() {
        let data =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/out.vx.txt"
            ))
            .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().collect::<Vec<_>>();
        let sample_addr = "00:1f.3".parse().unwrap();
        let sample_cs =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/config"
            ))
            [..64].read_with(&mut 0, LE).unwrap();
        let sample = vec![
            Device::new(sample_addr, sample_cs),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn iter_once_size_256() {
        let data =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/out.vxxx.txt"
            ))
            .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().collect::<Vec<_>>();
        let sample_addr = "00:1f.3".parse().unwrap();
        let sample_cs =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/config"
            ))
            .read_with(&mut 0, LE).unwrap();
        let sample = vec![
            Device::new(sample_addr, sample_cs),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn iter_once_size_4096() {
        let data =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:2030/out.vxxxx.txt"
            ))
            .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().collect::<Vec<_>>();
        let sample_addr = "ae:00.0".parse().unwrap();
        let sample_cs =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:2030/config"
            ))
            .read_with(&mut 0, LE).unwrap();
        let sample = vec![
            Device::new(sample_addr, sample_cs),
        ];
        // assert_eq!(sample, result);
        assert_eq!(sample[0].header.header_type, result[0].header.header_type);
    }
}
