//! PCI IDs Database parser

use core::num::ParseIntError;
use core::str::FromStr;
use core::str;
use core::iter::FromIterator;
use std::collections::HashMap;

use thiserror::Error;

pub const PCI_IDS_PATH: &'static str = "/usr/share/hwdata/pci.ids";


/// Structured entry of pci.ids file
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum PciIdsEntry {
    Vendor(Vendor),
    Device(Vendor, Device),
    Subsystem(Vendor, Device, Subsystem),
    Class(Class),
    Subclass(Class, Subclass),
    ProgIf(Class, Subclass, ProgIf),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseLineError {
    #[error("line mismatch")]
    LineMatch,
    #[error("invalid hex value")]
    Hex(#[from] ParseIntError),
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Vendor {
    pub id: u16,
    pub name: String,
}
impl FromStr for Vendor {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, space, name) = (s.get(0..4), s.get(4..6), s.get(6..));
        if let (Some(id), Some("  "), Some(name)) = (id, space, name) {
            Ok(Self {
                id: u16::from_str_radix(id, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}


#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Device {
    pub id: u16,
    pub name: String,
}
impl FromStr for Device {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tab, id, space, name) = (s.get(0..1), s.get(1..5), s.get(5..7), s.get(7..));
        if let (Some("\t"), Some(id), Some("  "), Some(name)) = (tab, id, space, name) {
            Ok(Self {
                id: u16::from_str_radix(id, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Subsystem {
    pub vendor_id: u16,
    pub device_id: u16,
    pub name: String,
}
impl FromStr for Subsystem {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tab, vid, did, name) = (s.get(0..2), s.get(2..6), s.get(7..11), s.get(13..));
        if let (Some("\t\t"), Some(vid), Some(did), Some(name)) = (tab, vid, did, name) {
            Ok(Self {
                vendor_id: u16::from_str_radix(vid, 16)?,
                device_id: u16::from_str_radix(did, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}


#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Class {
    pub id: u8,
    pub name: String,
}
impl FromStr for Class {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (c, id, space, name) = (s.get(0..2), s.get(2..4), s.get(4..6), s.get(6..));
        if let (Some("C "), Some(id), Some("  "), Some(name)) = (c, id, space, name) {
            Ok(Self {
                id: u8::from_str_radix(id, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Subclass {
    pub id: u8,
    pub name: String,
}
impl FromStr for Subclass {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tab, id, space, name) = (s.get(0..1), s.get(1..3), s.get(3..5), s.get(5..));
        if let (Some("\t"), Some(id), Some("  "), Some(name)) = (tab, id, space, name) {
            Ok(Self {
                id: u8::from_str_radix(id, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct ProgIf {
    pub id: u8,
    pub name: String,
}
impl FromStr for ProgIf {
    type Err = ParseLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tab, id, space, name) = (s.get(0..2), s.get(2..4), s.get(4..6), s.get(6..));
        if let (Some("\t\t"), Some(id), Some("  "), Some(name)) = (tab, id, space, name) {
            Ok(Self {
                id: u8::from_str_radix(id, 16)?,
                name: name.to_string(),
            })
        } else {
            Err(ParseLineError::LineMatch)
        }
    }
}


/// An iterator through pci.ids database file
#[derive(Debug, Clone)] 
pub struct PciIds<'a> {
    lines: str::Lines<'a>,
    vendor: Option<Vendor>,
    device: Option<Device>,
    class: Option<Class>,
    subclass: Option<Subclass>,
}
impl<'a> PciIds<'a> {
    pub fn new(lines: str::Lines<'a>)  -> Self {
        Self { lines, vendor: None, device: None, class: None, subclass: None }
    }
}
impl Iterator for PciIds<'_> {
    type Item = PciIdsEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            //  Skip non-informative lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Vendors, devices and subsystems
            if let Ok(vendor) = line.parse::<Vendor>() {
                self.device = None;
                self.vendor = Some(vendor.clone());
                return Some(PciIdsEntry::Vendor(vendor));
            }
            if let (Ok(device), Some(vendor)) = (line.parse::<Device>(), &self.vendor) {
                self.device = Some(device.clone());
                return Some(PciIdsEntry::Device(vendor.clone(), device));
            }
            if let (Ok(subsystem), Some(device), Some(vendor))
                = (line.parse::<Subsystem>(), &self.device, &self.vendor)
            {
                return Some(PciIdsEntry::Subsystem(vendor.clone(), device.clone(), subsystem));
            }
            // Class Codes
            if let Ok(class) = line.parse::<Class>() {
                self.subclass = None;
                self.class = Some(class.clone());
                return Some(PciIdsEntry::Class(class));
            }
            if let (Ok(subclass), Some(class)) = (line.parse::<Subclass>(), &self.class) {
                self.subclass = Some(subclass.clone());
                return Some(PciIdsEntry::Subclass(class.clone(), subclass));
            }
            if let (Ok(prog_if), Some(subclass), Some(class))
                = (line.parse::<ProgIf>(), &self.subclass, &self.class)
            {
                return Some(PciIdsEntry::ProgIf(class.clone(), subclass.clone(), prog_if));
            }
        }
        None
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)] 
enum VdsKey {
    Vendor(u16),
    Device(u16, u16),
    Subsystem(u16, u16, u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)] 
enum CcKey {
    Class(u8),
    Subclass(u8, u8),
    ProgIf(u8, u8, u8),
}

/// Struct to store pciids devices DB
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct VendorDeviceSubsystem(HashMap<VdsKey, String>);

impl VendorDeviceSubsystem {
    pub fn lookup<D, S>(&self, vendor_id: u16, device_id: D, subsystem_id: S) -> Option<String>
    where
        D: Into<Option<u16>>,
        S: Into<Option<(u16, u16)>>,
    {
        (match (device_id.into(), subsystem_id.into()) {
            (Some(device_id), Some((sub_vendor_id, sub_device_id))) =>
                self.0.get(&VdsKey::Subsystem(vendor_id, device_id, sub_vendor_id, sub_device_id)),
            (Some(device_id), None) =>
                self.0.get(&VdsKey::Device(vendor_id, device_id)), 
            _ => self.0.get(&VdsKey::Vendor(vendor_id)), 
        }).cloned()
    }
}

pub struct ClassCode(HashMap<CcKey, String>);

impl ClassCode {
    pub fn lookup<S, P>(&self, class_id: u8, subclass_id: S, prog_if_id: P) -> Option<String>
    where
        S: Into<Option<u8>>,
        P: Into<Option<u8>>,
    {
        (match (subclass_id.into(), prog_if_id.into()) {
            (Some(subclass_id), Some(prog_if_id)) =>
                self.0.get(&CcKey::ProgIf(class_id, subclass_id, prog_if_id)),
            (Some(subclass_id), None) =>
                self.0.get(&CcKey::Subclass(class_id, subclass_id)),
            _ => self.0.get(&CcKey::Class(class_id)),
        }).cloned()
    }
}

impl FromIterator<PciIdsEntry> for (VendorDeviceSubsystem, ClassCode) {
    fn from_iter<I: IntoIterator<Item = PciIdsEntry>>(iter: I) -> Self {
        let mut vds = HashMap::default();
        let mut cc = HashMap::default();
        for entry in iter {
            match entry {
                PciIdsEntry::Vendor(vendor) =>
                    vds.insert(VdsKey::Vendor(vendor.id), vendor.name),
                PciIdsEntry::Device(vendor, device) =>
                    vds.insert(VdsKey::Device(vendor.id, device.id), device.name),
                PciIdsEntry::Subsystem(vendor, device, subsystem) => {
                    let key = VdsKey::Subsystem(
                        vendor.id, device.id, subsystem.vendor_id, subsystem.device_id
                    );
                    vds.insert(key, subsystem.name)
                },
                PciIdsEntry::Class(class) =>
                    cc.insert(CcKey::Class(class.id), class.name),
                PciIdsEntry::Subclass(class, subclass) =>
                    cc.insert(CcKey::Subclass(class.id, subclass.id), subclass.name),
                PciIdsEntry::ProgIf(class, subclass, prog_if) =>
                    cc.insert(CcKey::ProgIf(class.id, subclass.id, prog_if.id), prog_if.name),
            };
        }
        (VendorDeviceSubsystem(vds), ClassCode(cc))
    }
}



#[cfg(test)]
mod tests {
    use std::prelude::v1::*;
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn parse_vendor_valid() {
        let result = "aa55  Ncomputing X300 PCI-Engine".parse::<Vendor>().unwrap();
        assert_eq!(Vendor { id: 0xaa55, name: "Ncomputing X300 PCI-Engine".into() }, result);
    }

    #[test]
    fn parse_vendor_invalid_match() {
        let result = "#aa55  Ncomputing X300 PCI-Engine".parse::<Vendor>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_vendor_invalid_hex() {
        let result = "ag55  Ncomputing X300 PCI-Engine".parse::<Vendor>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u16::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn parse_device_valid() {
        let result = "	1601  AimTrak".parse::<Device>().unwrap();
        assert_eq!(Device { id: 0x1601, name: "AimTrak".into() }, result);
    }

    #[test]
    fn parse_device_invalid_match() {
        let result = "1601  AimTrak".parse::<Device>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_device_invalid_hex() {
        let result = "	160z  AimTrak".parse::<Device>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u16::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn parse_subsystem_valid() {
        let result = "		b1d9 0003  AX400P 4-port analog card"
            .parse::<Subsystem>().unwrap();
        let sample = Subsystem {
            vendor_id: 0xb1d9,
            device_id: 0x0003,
            name: "AX400P 4-port analog card".into()
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn parse_subsystem_invalid_match() {
        let result = "	b1d9 0003  AX400P 4-port analog card"
            .parse::<Subsystem>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_subsystem_invalid_hex() {
        let result = "		bZd9 0003  AX400P 4-port analog card"
            .parse::<Subsystem>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u16::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn parse_class_valid() {
        let result = "C 06  Bridge".parse::<Class>().unwrap();
        assert_eq!(Class { id: 0x06, name: "Bridge".into() }, result);
    }

    #[test]
    fn parse_class_invalid_match() {
        let result = "02ac  SpeedStream".parse::<Class>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_class_invalid_hex() {
        let result = "C Z6  Bridge".parse::<Class>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u8::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn parse_subclass_valid() {
        let result = "	01  Satellite TV controller".parse::<Subclass>().unwrap();
        assert_eq!(Subclass { id: 0x01, name: "Satellite TV controller".into() }, result);
    }

    #[test]
    fn parse_subclass_invalid_match() {
        let result = "	0ccd  CCD-CALYPSO".parse::<Subclass>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_subclass_invalid_hex() {
        let result = "	0Z  Satellite TV controller".parse::<Subclass>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u16::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn parse_progif_valid() {
        let result = "		02  BT (Block Transfer)".parse::<ProgIf>().unwrap();
        assert_eq!(ProgIf { id: 0x02, name: "BT (Block Transfer)".into() }, result);
    }

    #[test]
    fn parse_progif_invalid_match() {
        let result = "		001c 0004  2 Channel CAN Bus SJC1000".parse::<ProgIf>().unwrap_err();
        assert_eq!(ParseLineError::LineMatch, result);
    }

    #[test]
    fn parse_progif_invalid_hex() {
        let result = "		ZZ  BT (Block Transfer)".parse::<ProgIf>().unwrap_err();
        assert_eq!(ParseLineError::Hex(u16::from_str_radix("z", 2).unwrap_err()), result);
    }

    #[test]
    fn pciids_iterator() {
        let data = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/tests/data/pci.ids"
        ));
        let pciids = PciIds::new(data.lines());
        let mut total = 0;
        let mut miss = 0;
        for entry in pciids {
            total += 1;
            match entry {
                // First VDS entry
                PciIdsEntry::Vendor(Vendor { id: 0x0001, name }) =>
                    assert_eq!(name, "SafeNet (wrong ID)"),
                // In the middle
                PciIdsEntry::Subsystem(
                    Vendor { id: 0x1371, .. },
                    Device { id: 0x434e, .. },
                    Subsystem { vendor_id: 0x1371, device_id: 0x434e, name }
                ) => assert_eq!(name, "N-Way PCI-Bus Giga-Card 1000/100/10Mbps(L)"),
                // Last VDS entry
                PciIdsEntry::Vendor(Vendor { id: 0xffff, name }) =>
                    assert_eq!(name, "Illegal Vendor ID"),
                // First Class Code
                PciIdsEntry::Class(Class { id: 0x00, name }) =>
                    assert_eq!(name, "Unclassified device"),
                // Somethere in the middle
                PciIdsEntry::ProgIf(
                    Class { id: 0x09, .. },
                    Subclass { id: 0x04, .. },
                    ProgIf { id: 0x10, name }
                ) => assert_eq!(name, "Extended"),
                // Last Class Code
                PciIdsEntry::Class(Class { id: 0xff, name }) =>
                    assert_eq!(name, "Unassigned class"),
                _ => miss += 1,
            }
        }
        // Check all meaningful matches asserted
        assert_eq!(total - 6, miss)
    }

    #[test]
    fn collect_pciids_vds_db() {
        let data = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/tests/data/pci.ids"
        ));
        let pciids = PciIds::new(data.lines());
        let (vds, _) = pciids.collect::<(VendorDeviceSubsystem, _)>();
        assert_eq!(
            vds.lookup(0x0010, None, None).unwrap(),
            "Allied Telesis, Inc (Wrong ID)"
        );
        assert_eq!(
            vds.lookup(0x0010, 0x8139, None).unwrap(),
            "AT-2500TX V3 Ethernet"
        );
        assert_eq!(
            vds.lookup(0x001c, 0x0001, (0x001c, 0x0004)).unwrap(),
            "2 Channel CAN Bus SJC1000"
        );
        assert_eq!(
            vds.lookup(0x0000, 0x0000, (0x0000, 0x0000)),
            None
        );
    }

    #[test]
    fn collect_pciids_cc_db() {
        let data = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/tests/data/pci.ids"
        ));
        let pciids = PciIds::new(data.lines());
        let (_, cc) = pciids.collect::<(_, ClassCode)>();
        assert_eq!(
            cc.lookup(0x01, None, None).unwrap(),
            "Mass storage controller"
        );
        assert_eq!(
            cc.lookup(0x01, 0x01, None).unwrap(),
            "IDE interface"
        );
        assert_eq!(
            cc.lookup(0x01, 0x01, 0x05).unwrap(),
            "PCI native mode-only controller"
        );
        assert_eq!(
            cc.lookup(0xaa, 0xaa, 0xaa),
            None
        );
    }
}
