use core::{
    fmt::{self, Display},
    num::ParseIntError,
    str::FromStr,
};

use thiserror::Error;


#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Address {
    /// DOMAIN/SEGMENT is primarily a PLATFORM level construct. Logically, DOMAIN is the most
    /// significant selector (most significant address bits selector) in the
    /// DOMAIN:Bus:Device:Function:Offset addressing scheme of the PCI Family Configuration Space
    /// addressing mechanism.
    pub domain: u16,
    /// The PCI specification permits a single system to host up to 256 buses
    pub bus: u8,
    /// Each bus hosts up to 32 devices
    pub device: u8,
    /// Each device can be a multifunction board (such as an audio device withan accompanying
    /// CD-ROM drive) with a maximum of eight functions. 
    pub function: u8,
}

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum ParseAddressError {
    #[error("empty string")]
    Empty,
    #[error("no dot between device and function")]
    NoDot,
    #[error("no colon between bus and device")]
    NoColon,
    #[error("domain parsing problem")]
    Domain(#[source] ParseIntError),
    #[error("bus parsing problem")]
    Bus(#[source] ParseIntError),
    #[error("device parsing problem")]
    Device(#[source] ParseIntError),
    #[error("function parsing problem")]
    Function(#[source] ParseIntError),
    #[error("invalid device number {0} (only 32 available)")]
    DeviceNumber(u8),
    #[error("invalid function number {0} (only 8 available)")]
    FunctionNumber(u8),
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { domain: dom, bus: b, device: dev, function: fun } = self;
        if f.alternate() && self.domain == 0 {
            write!(f, "{:02x}:{:02x}.{:x}", b, dev, fun)
        } else {
            write!(f, "{:04x}:{:02x}:{:02x}.{:x}", dom, b, dev, fun)
        }
    }
}
impl FromStr for Address {
    type Err = ParseAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseAddressError::Empty);
        }
        // Domain may be absent so we will iterate from the end
        // Function
        let (s, function) = s.rsplit_once('.')
            .ok_or(ParseAddressError::NoDot)?;
        let function = u8::from_str_radix(function, 16)
            .map_err(ParseAddressError::Function)?;
        if function > 7 {
            return Err(ParseAddressError::FunctionNumber(function));
        }
        // Device
        let (s, device) = s.rsplit_once(':')
            .ok_or(ParseAddressError::NoColon)?;
        let device = u8::from_str_radix(device, 16)
            .map_err(ParseAddressError::Device)?;
        if device > 31 {
            return Err(ParseAddressError::DeviceNumber(device));
        }
        // Domain and Bus
        let (domain, s) =
            if let Some((domain, s)) = s.split_once(':') {
                // Domain
                let domain = u16::from_str_radix(domain, 16)
                    .map_err(ParseAddressError::Domain)?;
                (domain, s)
            } else {
                (0, s)
            };
        // Bus
        let bus = u8::from_str_radix(s, 16)
            .map_err(ParseAddressError::Bus)?;
        Ok(Self { domain, bus, device, function })
    }
}



#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn parse_address() {
        let data = [
            (Ok(Address { domain: 0x0000, bus: 0x00, device: 0x14, function: 0x03 }), "0000:00:14.3"),
            (Ok(Address { domain: 0x0000, bus: 0x00, device: 0x14, function: 0x03 }), "00:14.3"),
            (Err(ParseAddressError::Empty), ""),
            (Err(ParseAddressError::NoDot), "00"),
            (Err(ParseAddressError::NoColon), "00.0"),
            (Err(ParseAddressError::Function(u8::from_str_radix("x", 16).unwrap_err())), "00:00.x"),
            (Err(ParseAddressError::Device(u8::from_str_radix("x", 16).unwrap_err())), "00:xx.0"),
            (Err(ParseAddressError::Bus(u8::from_str_radix("x", 16).unwrap_err())), "xx:00.0"),
            (Err(ParseAddressError::Domain(u16::from_str_radix("x", 16).unwrap_err())), "xxxx:00:00.0"),
            (Err(ParseAddressError::FunctionNumber(0xAA)), "00:00.AA"),
            (Err(ParseAddressError::DeviceNumber(0xAA)), "00:AA.0"),
        ];
        for (n, (sample, s)) in data.iter().enumerate() {
            let result = s.parse();
            assert_eq!(sample, &result, "#{}", n);
        }
    }
}
