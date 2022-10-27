use std::{collections::HashMap, fs, io, num::ParseIntError, path::Path, str::FromStr};

mod pciids;
use pciids::PciIds;

mod hwdb;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Names {
    vendor_device_subsystem: VendorDeviceSubsystem,
    class_code: ClassCode,
}

impl Names {
    pub fn init() -> io::Result<Self> {
        Self::init_hwdb().or_else(|_| Self::init_pciids(pciids::PciIds::PATH))
    }
    pub fn init_hwdb() -> io::Result<Self> {
        let vds = fs::read_to_string(hwdb::VendorModel::PATH)?;
        let vendor_device_subsystem = hwdb::VendorModel::new(&vds).collect();
        let cc = fs::read_to_string(hwdb::Classes::PATH)?;
        let class_code = hwdb::Classes::new(&cc).collect();
        Ok(Self {
            vendor_device_subsystem,
            class_code,
        })
    }
    pub fn init_pciids(path: impl AsRef<Path>) -> io::Result<Self> {
        fs::read_to_string(path.as_ref()).map(|s| {
            let (vendor_device_subsystem, class_code) = PciIds::new(s.lines()).collect();
            Self {
                vendor_device_subsystem,
                class_code,
            }
        })
    }
    pub fn vendor_device_subsystem(&self) -> VendorDeviceSubsystem {
        self.vendor_device_subsystem.clone()
    }
    pub fn class_code(&self) -> ClassCode {
        self.class_code.clone()
    }
}

/// Struct to store pciids devices DB
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VendorDeviceSubsystem(pub HashMap<VdsKey, String>);

impl VendorDeviceSubsystem {
    pub fn lookup<V, D, S>(&self, vendor_id: V, device_id: D, subsystem_id: S) -> Option<String>
    where
        V: Into<Option<u16>>,
        D: Into<Option<u16>>,
        S: Into<Option<(u16, u16)>>,
    {
        let data = &self.0;
        let name = match (vendor_id.into(), device_id.into(), subsystem_id.into()) {
            // Lookup "generic" subsystem
            (None, None, Some((sv, sd))) => data.iter().find_map(|(k, v)| {
                if let &VdsKey::Subsystem(_, _, sv_, sd_) = k {
                    if sv == sv_ && sd == sd_ {
                        Some(v)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            (Some(v), Some(d), Some((sv, sd))) => data.get(&VdsKey::Subsystem(v, d, sv, sd)),
            (Some(v), Some(d), None) => data.get(&VdsKey::Device(v, d)),
            (Some(v), _, _) => data.get(&VdsKey::Vendor(v)),
            _ => None,
        };
        name.cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VdsKey {
    Vendor(u16),
    Device(u16, u16),
    Subsystem(u16, u16, u16, u16),
}

impl From<Modalias> for VdsKey {
    fn from(modalias: Modalias) -> Self {
        match modalias {
            Modalias {
                vendor: Some(v),
                device: Some(d),
                sub_vendor: Some(sv),
                sub_device: Some(sd),
                ..
            } => Self::Subsystem(v as u16, d as u16, sv as u16, sd as u16),
            Modalias {
                vendor: Some(v),
                device: Some(d),
                sub_vendor: None,
                sub_device: None,
                ..
            } => Self::Device(v as u16, d as u16),
            Modalias {
                vendor: Some(v),
                device: None,
                sub_vendor: None,
                sub_device: None,
                ..
            } => Self::Vendor(v as u16),
            _ => Self::Vendor(0),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ClassCode(pub HashMap<CcKey, String>);

impl ClassCode {
    pub fn lookup<S, P>(&self, class_id: u8, subclass_id: S, prog_if_id: P) -> Option<String>
    where
        S: Into<Option<u8>>,
        P: Into<Option<u8>>,
    {
        (match (subclass_id.into(), prog_if_id.into()) {
            (Some(subclass_id), Some(prog_if_id)) => {
                self.0
                    .get(&CcKey::ProgIf(class_id, subclass_id, prog_if_id))
            }
            (Some(subclass_id), None) => self.0.get(&CcKey::Subclass(class_id, subclass_id)),
            _ => self.0.get(&CcKey::Class(class_id)),
        })
        .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CcKey {
    Class(u8),
    Subclass(u8, u8),
    ProgIf(u8, u8, u8),
}

impl From<Modalias> for CcKey {
    fn from(modalias: Modalias) -> Self {
        match modalias {
            Modalias {
                base_class: Some(bc),
                sub_class: Some(sc),
                interface: Some(i),
                ..
            } => Self::ProgIf(bc, sc, i),
            Modalias {
                base_class: Some(bc),
                sub_class: Some(sc),
                interface: None,
                ..
            } => Self::Subclass(bc, sc),
            Modalias {
                base_class: Some(bc),
                sub_class: None,
                interface: None,
                ..
            } => Self::Class(bc),
            _ => Self::Class(0),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModaliasError {
    #[error("prefix should be equal to 'pci:'")]
    Prefix,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Modalias {
    vendor: Option<u32>,
    device: Option<u32>,
    sub_vendor: Option<u32>,
    sub_device: Option<u32>,
    base_class: Option<u8>,
    sub_class: Option<u8>,
    interface: Option<u8>,
}

impl FromStr for Modalias {
    type Err = ModaliasError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("pci:").ok_or(ModaliasError::Prefix)?;
        let parse_u32 = |pattern| -> Option<u32> {
            s.split_once(pattern).and_then(|(_, s)| {
                let dword = s.get(..8)?;
                u32::from_str_radix(dword, 16).ok()
            })
        };
        let parse_u8 = |pattern| -> Option<u8> {
            s.split_once(pattern).and_then(|(_, s)| {
                let dword = s.get(..2)?;
                u8::from_str_radix(dword, 16).ok()
            })
        };
        Ok(Self {
            vendor: parse_u32("v"),
            device: parse_u32("d"),
            sub_vendor: parse_u32("sv"),
            sub_device: parse_u32("sd"),
            base_class: parse_u8("bc"),
            sub_class: parse_u8("sc"),
            interface: parse_u8("i"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_valid_vds_modalias() {
        let data = [
            "pci:v0000F5F5*",
            "pci:v00009005d00000286sv00009005sd00000800*",
            "pci:v00009004d00007896*",
        ];
        let result = data.map(|s| s.parse());
        let sample = [
            Ok(Modalias {
                vendor: Some(0xf5f5),
                device: None,
                sub_vendor: None,
                sub_device: None,
                base_class: None,
                sub_class: None,
                interface: None,
            }),
            Ok(Modalias {
                vendor: Some(0x9005),
                device: Some(0x0286),
                sub_vendor: Some(0x9005),
                sub_device: Some(0x0800),
                base_class: None,
                sub_class: None,
                interface: None,
            }),
            Ok(Modalias {
                vendor: Some(0x9004),
                device: Some(0x7896),
                sub_vendor: None,
                sub_device: None,
                base_class: None,
                sub_class: None,
                interface: None,
            }),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn parse_valid_cc_modalias() {
        let data = [
            "pci:v*d*sv*sd*bc05*",
            "pci:v*d*sv*sd*bc05sc02*",
            "pci:v*d*sv*sd*bc08sc03i01*",
        ];
        let result = data.map(|s| s.parse());
        let sample = [
            Ok(Modalias {
                vendor: None,
                device: None,
                sub_vendor: None,
                sub_device: None,
                base_class: Some(0x05),
                sub_class: None,
                interface: None,
            }),
            Ok(Modalias {
                vendor: None,
                device: None,
                sub_vendor: None,
                sub_device: None,
                base_class: Some(0x05),
                sub_class: Some(0x02),
                interface: None,
            }),
            Ok(Modalias {
                vendor: None,
                device: None,
                sub_vendor: None,
                sub_device: None,
                base_class: Some(0x08),
                sub_class: Some(0x03),
                interface: Some(0x01),
            }),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn parse_invalid_modalias() {
        let data = [
            "",
            "\n",
            "# This file is part of systemd.",
            " ID_VENDOR_FROM_DATABASE=Tiger Jet Network Inc. (Wrong ID)",
        ];
        let result = data.map(Modalias::from_str);
        let sample = [
            Err(ModaliasError::Prefix),
            Err(ModaliasError::Prefix),
            Err(ModaliasError::Prefix),
            Err(ModaliasError::Prefix),
        ];
        assert_eq!(sample, result);
    }
}
