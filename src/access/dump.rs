use std::{
    fs,
    iter::{Enumerate, Peekable},
    num::ParseIntError,
    path::PathBuf,
    str::{FromStr, Lines},
};

use thiserror::Error;

use crate::{
    access::AccessMethod,
    device::{
        address::ParseAddressError, Address, ConfigurationSpace, Device, DeviceDependentRegion,
        ExtendedConfigurationSpace,
    },
};

use super::AccessError;

#[derive(Error, Clone, Eq, PartialEq, Debug)]
#[error("malformed line #{line}: {source}")]
pub struct DumpError {
    line: usize,
    source: LineError,
}

#[derive(Error, Clone, Eq, PartialEq, Debug)]
pub enum LineError {
    #[error(transparent)]
    Address(#[from] AddressLineError),
    #[error(transparent)]
    Hex(#[from] HexLineError),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Dump {
    content: String,
}

impl Dump {
    pub fn new(s: impl ToString) -> Self {
        Self {
            content: s.to_string(),
        }
    }
    pub fn init(path: impl Into<PathBuf>) -> super::Result<Self> {
        let path = path.into();
        fs::read_to_string(&path)
            .map(|s| Self { content: s })
            .map_err(|source| AccessError::File { path, source })
    }
}

impl<'a> AccessMethod<'a> for Dump {
    type Scan = Scan<Iter<'a>>;
    type Iter = Iter<'a>;

    fn scan(&'a self) -> Self::Scan {
        Scan::new(self.iter())
    }

    fn iter(&'a self) -> Self::Iter {
        Iter::new(self.content.lines())
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum AddressLineError {
    #[error("empty line")]
    Empty,
    #[error("address pattern should be 7, 12 or 13 chars, not {0}")]
    AddressPatternLength(usize),
    #[error(transparent)]
    Address(#[from] ParseAddressError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AddressLine(Address);

impl FromStr for AddressLine {
    type Err = AddressLineError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let field = line
            .split_ascii_whitespace()
            .next()
            .ok_or(AddressLineError::Empty)?;
        let len = field.len();
        if !(len == 7 || len == 12 || len == 13) {
            return Err(AddressLineError::AddressPatternLength(len));
        }
        let address = field.parse()?;
        Ok(Self(address))
    }
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum HexLineError {
    #[error("malformed offset")]
    OffsetPattern,
    #[error("offset {0:#04x} should be <= 0xff0")]
    OffsetOverflow(usize),
    #[error("byte #{number:#02x}: {source}")]
    ParseInterror {
        number: usize,
        source: ParseIntError,
    },
}

// Hex dump line looks like:
// 20: 00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15
#[derive(Clone, Debug, PartialEq, Eq)]
struct HexLine {
    offset: usize,
    u8x16: [u8; 16],
}

impl FromStr for HexLine {
    type Err = HexLineError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut fields = line.split_ascii_whitespace();
        let offset = fields
            .next()
            .and_then(|s| {
                let s = s.strip_suffix(':')?;
                let len = s.len();
                if len == 2 || len == 3 {
                    usize::from_str_radix(s, 16).ok()
                } else {
                    None
                }
            })
            .ok_or(HexLineError::OffsetPattern)?;
        if offset > 0xff0 {
            return Err(HexLineError::OffsetOverflow(offset));
        }
        let mut u8x16 = [0u8; 16];
        for (number, (byte, s)) in &mut u8x16.iter_mut().zip(fields).enumerate() {
            *byte = u8::from_str_radix(s, 16)
                .map_err(|source| HexLineError::ParseInterror { number, source })?;
        }
        Ok(HexLine { offset, u8x16 })
    }
}

pub struct Iter<'a> {
    lines: Peekable<Enumerate<Lines<'a>>>,
}

impl<'a> Iter<'a> {
    pub fn new(lines: Lines<'a>) -> Self {
        Self {
            lines: lines.enumerate().peekable(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = super::Result<Device>;
    fn next(&mut self) -> Option<Self::Item> {
        let AddressLine(address) = self.lines.find_map(|(_, line)| line.parse().ok())?;

        let mut buf = [0u8; 4096];
        let mut end = 0;
        while let Some((n, line)) = self
            .lines
            .next_if(|(_, line)| line.parse::<AddressLine>().is_err())
        {
            // Non-parsible lines ommit
            match line.parse() {
                Ok(HexLine { offset, u8x16 }) => {
                    // It is safe to use index, because we checked it in step below
                    let buf = buf[offset..].iter_mut();
                    for (dst, src) in &mut buf.zip(u8x16.iter()) {
                        *dst = *src;
                    }
                    end = end.max(offset);
                }
                Err(HexLineError::OffsetPattern) => {
                    continue;
                }
                Err(err) => {
                    let dump_error = DumpError {
                        line: n,
                        source: err.into(),
                    };
                    return Some(Err(dump_error.into()));
                }
            }
        }
        let end = match end {
            0..=63 => DeviceDependentRegion::OFFSET,
            64..=255 => ExtendedConfigurationSpace::OFFSET,
            _ => ConfigurationSpace::SIZE,
        };
        let result = buf[..end]
            .try_into()
            .map(|cs| Device::new(address, cs))
            .map_err(|_| AccessError::ConfigurationSpace);
        Some(result)
    }
}

pub struct Scan<I> {
    iter: I,
}

impl<I> Scan<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator<Item = super::Result<Device>>> Iterator for Scan<I> {
    type Item = super::Result<Address>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|result| result.map(|Device { address, .. }| address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn address_line_valid_short() {
        let AddressLine(result) =
            "02:00.0 3D controller: NVIDIA Corporation GP108M [GeForce MX150] (rev a1)"
                .parse()
                .unwrap();
        let sample: Address = "02:00.0".parse().unwrap();
        assert_eq!(sample, result);
    }

    #[test]
    fn address_line_valid_full() {
        let AddressLine(result) = "0123:02:00.0 _".parse().unwrap();
        let sample: Address = "0123:02:00.0".parse().unwrap();
        assert_eq!(sample, result);
    }

    #[test]
    fn address_line_invalid_empty() {
        let result = "".parse::<AddressLine>().unwrap_err();
        assert_eq!(AddressLineError::Empty, result);
    }

    #[test]
    fn address_line_invalid_address_pattern_length() {
        let result = "012345678 _".parse::<AddressLine>().unwrap_err();
        assert_eq!(AddressLineError::AddressPatternLength(9), result);
    }

    #[test]
    fn address_line_invalid_parse() {
        let result = "XXXX:XX:XX.X _".parse::<AddressLine>().unwrap_err();
        assert!(
            matches!(result, AddressLineError::Address(_)),
            "{:?}",
            result
        );
    }

    #[test]
    fn hex_line_valid() {
        let result = "20: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f".parse();
        let sample = HexLine {
            offset: 0x20,
            u8x16: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        };
        assert_eq!(Ok(sample), result);
    }

    #[test]
    fn hex_line_invalid_offset_malformed() {
        let result = "20 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f".parse::<HexLine>();
        assert_eq!(result, Err(HexLineError::OffsetPattern));
    }

    #[test]
    fn hex_line_invalid_offset_overflow() {
        let result = "ff1: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f".parse::<HexLine>();
        assert_eq!(result, Err(HexLineError::OffsetOverflow(0xff1)));
    }

    #[test]
    fn hex_line_invalid_values() {
        let result = "20: 00 01 02 03 04 Y5 06 07 08 09 0a 0b 0c 0d 0e 0f".parse::<HexLine>();
        assert!(matches!(
            result,
            Err(HexLineError::ParseInterror { number: 5, .. })
        ));
    }

    #[test]
    fn dump_init_valid() {
        let result = Dump::init(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/out.vx.txt"
        ))
        .unwrap();
        let sample = Dump {
            content: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/out.vx.txt"
            ))
            .to_string(),
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn dump_init_invalid() {
        let path = "/b3be10339da4a12e22bdc481cba1b03018ba894332550b194e8aa32ae93d7fa3";
        let result = Dump::init(path).unwrap_err();
        use std::error::Error;
        assert_str_eq!(
            "No such file or directory (os error 2)",
            result.source().unwrap().to_string()
        );
    }

    #[test]
    fn iter_once_size_64() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/out.vx.txt"
        ))
        .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().next().unwrap();
        let sample_addr = "00:1f.3".parse().unwrap();
        let sample_cs = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/config"
        ))[..64]
            .try_into()
            .unwrap();
        let sample = Device::new(sample_addr, sample_cs);
        assert_eq!(Ok(sample), result);
    }

    #[test]
    fn iter_once_size_256() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/out.vxxx.txt"
        ))
        .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().map(Result::unwrap).collect::<Vec<_>>();
        let sample_addr = "00:1f.3".parse().unwrap();
        let sample_cs = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/config"
        ))
        .as_slice()
        .try_into()
        .unwrap();
        let sample = vec![Device::new(sample_addr, sample_cs)];
        assert_eq!(sample, result);
    }

    #[test]
    fn iter_once_size_4096() {
        let data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:2030/out.vxxxx.txt"
        ))
        .to_string();
        let dump = Dump::new(data);
        let result = dump.iter().map(Result::unwrap).collect::<Vec<_>>();
        let sample_addr = "ae:00.0".parse().unwrap();
        let sample_cs = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:2030/config"
        ))
        .as_slice()
        .try_into()
        .unwrap();
        let sample = vec![Device::new(sample_addr, sample_cs)];
        // assert_eq!(sample, result);
        assert_eq!(sample[0].header.header_type, result[0].header.header_type);
    }

    #[test]
    fn scan() {
        let addr: Address = "00:1f.3".parse().unwrap();
        let cs = [0u8; 64].as_slice().try_into().unwrap();
        let device = Device::new(addr.clone(), cs);
        let mut scan = Scan::new([Ok(device)].into_iter());
        let result = scan.next();
        assert_eq!(Some(Ok(addr)), result);
    }
}
