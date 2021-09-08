use core::str;
use core::iter::FromIterator;
use std::collections::HashMap;

pub const PCI_IDS_PATH: &'static str = "/usr/share/hwdata/pci.ids";

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Vendor<'a> {
    pub id: u16,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Device<'a> {
    pub id: u16,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct SubDevice<'a> {
    pub vendor_id: u16,
    pub device_id: u16,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct PciDevice<'a> {
    pub vendor: Vendor<'a>,
    pub device: Option<Device<'a>>,
    pub sub_device: Option<SubDevice<'a>>,
}

/// An iterator through Vendor, Device, SubDevice triples
#[derive(Debug, Clone)]
pub struct PciDevices<'a> {
    lines: str::Lines<'a>,
    vendor: Option<Vendor<'a>>,
    device: Option<Device<'a>>,
}

/// Struct to store pciids devices DB
#[derive(Debug, Clone, Default)]
pub struct PciidsDevices {
    pub vendors: HashMap<u16, String>,
    pub devices: HashMap<(u16, u16), String>,
    pub sub_devices: HashMap<(u16, u16), String>,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Class<'a> {
    pub id: u8,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct SubClass<'a> {
    pub id: u8,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct ProgIf<'a> {
    pub id: u8,
    pub name: &'a str,
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct ClassCode<'a> {
    pub class: Class<'a>,
    pub sub_class: Option<SubClass<'a>>,
    pub prog_if: Option<ProgIf<'a>>,
}

/// An iterator through device's type, the device's function, and the device's register-level
/// programming interface triples
#[derive(Debug, Clone)]
pub struct ClassCodes<'a> {
    lines: str::Lines<'a>,
    class: Option<Class<'a>>,
    sub_class: Option<SubClass<'a>>,
}



impl<'a> PciDevice<'a> {
    pub fn new(vendor: Vendor<'a>, device: Option<Device<'a>>, sub_device: Option<SubDevice<'a>>)  -> Self {
        Self { vendor, device, sub_device }
    }
}

impl<'a> PciDevices<'a> {
    pub fn new(data: &'a str)  -> Self {
        Self { lines: data.lines() , vendor: None, device: None }
    }
}
impl<'a> Iterator for PciDevices<'a> {
    type Item = PciDevice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            if line.is_empty() || line.starts_with('#') || line.starts_with('C') {
                continue;
            }
            if let (true, Some(device), Some(vendor)) = (line.starts_with("\t\t"), &self.device, &self.vendor) {
                let mut words = line.trim().splitn(3, " ");
                let subdevice = (|| {
                    let vendor_id = u16::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let device_id = u16::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let name = words.next()?.trim();
                    Some(SubDevice { vendor_id, device_id, name })
                })();
                return Some(PciDevice::new(vendor.clone(), Some(device.clone()), subdevice));
            }
            if let (true, Some(vendor)) = (line.starts_with("\t"), &self.vendor) {
                let mut words = line.trim().splitn(2, " ");
                let device = (|| {
                    let id = u16::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let name = words.next()?.trim();
                    Some(Device { id, name })
                })();
                self.device = device.clone();
                return Some(PciDevice::new(vendor.clone(), device, None));
            }
            let mut words = line.trim().splitn(2, " ");
            let vendor = (|| {
                let id = u16::from_str_radix(words.next()?.trim(), 16).ok()?;
                let name = words.next()?.trim();
                Some(Vendor { id, name })
            })();
            self.vendor = vendor.clone();
            return vendor.map(|vendor| PciDevice::new(vendor, None, None));
        }
        None
    }
}
impl<'a> FromIterator<PciDevice<'a>> for PciidsDevices {
    fn from_iter<I: IntoIterator<Item = PciDevice<'a>>>(iter: I) -> Self {
        let mut result = Self::default();
        for PciDevice { vendor, device, sub_device } in iter {
            result.vendors
                .entry(vendor.id)
                .or_insert(vendor.name.to_string());
            if let Some(device) = device {
                result.devices
                    .entry((vendor.id, device.id))
                    .or_insert(device.name.to_string());
            }
            if let Some(sub_device) = sub_device {
                result.sub_devices
                    .entry((sub_device.vendor_id, sub_device.device_id))
                    .or_insert(sub_device.name.to_string());
            }
        }
        result
    }
}

impl<'a> ClassCode<'a> {
    pub fn new(class: Class<'a>, sub_class: Option<SubClass<'a>>, prog_if: Option<ProgIf<'a>>)  -> Self {
        Self { class, sub_class, prog_if }
    }
}

impl<'a> ClassCodes<'a> {
    pub fn new(data: &'a str)  -> Self {
        Self { lines: data.lines() , class: None, sub_class: None }
    }
}
impl<'a> Iterator for ClassCodes<'a> {
    type Item = ClassCode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let (true, Some(sub_class), Some(class)) = (line.starts_with("\t\t"), &self.sub_class, &self.class) {
                let mut words = line.trim().splitn(2, " ");
                let prog_if = (|| {
                    let id = u8::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let name = words.next()?.trim();
                    Some(ProgIf { id, name })
                })();
                return Some(ClassCode::new(class.clone(), Some(sub_class.clone()), prog_if));
            }
            if let (true, Some(class)) = (line.starts_with("\t"), &self.class) {
                let mut words = line.trim().splitn(2, " ");
                let sub_class = (|| {
                    let id = u8::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let name = words.next()?.trim();
                    Some(SubClass { id, name })
                })();
                self.sub_class = sub_class.clone();
                return Some(ClassCode::new(class.clone(), sub_class, None));
            }
            if let Some(line) = line.strip_prefix("C ") {
                let mut words = line.splitn(2, " ");
                let class = (|| {
                    let id = u8::from_str_radix(words.next()?.trim(), 16).ok()?;
                    let name = words.next()?.trim();
                    Some(Class { id, name })
                })();
                self.class = class.clone();
                return class.map(|class| ClassCode::new(class, None, None));
            }
        }
        None
    }
}



#[cfg(test)]
mod tests {
    use std::{prelude::v1::*};
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn pci_devices() {
        let data = include_str!("/usr/share/hwdata/pci.ids");
        let mut pci_devices = PciDevices::new(data);
        let i350_t2_result = pci_devices
            .find(|PciDevice { vendor, device, sub_device }| {
                vendor.id == 0x8086 && 
                device.as_ref().map(|d| d.id) == Some(0x1521) &&
                sub_device.as_ref().map(|sd| (sd.vendor_id, sd.device_id)) == Some((0x8086, 0x0002))
            })
            .unwrap();
        let i350_t2_sample = PciDevice {
            vendor: Vendor { id: 0x8086, name: "Intel Corporation" },
            device: Some(Device { id: 0x1521, name: "I350 Gigabit Network Connection" }),
            sub_device: Some(SubDevice { vendor_id: 0x8086, device_id: 0x0002, name: "Ethernet Server Adapter I350-T2" }),
        };
        assert_eq!(i350_t2_sample, i350_t2_result, "Ethernet Server Adapter I350-T2");
    }

    #[test]
    fn pciids_devices() {
        let data = include_str!("/usr/share/hwdata/pci.ids");
        let db: PciidsDevices = PciDevices::new(data).collect();
        
        assert_eq!(db.vendors.get(&0x0357), Some(&"TTTech Computertechnik AG (Wrong ID)".to_string()));
        assert_eq!(db.vendors.get(&0xfff0), None, "Non-existing vendor");
        
        assert_eq!(db.devices.get(&(0x1017,0x5343)), Some(&"SPEA 3D Accelerator".to_string()));
        assert_eq!(db.devices.get(&(0xfff0,0xfff0)), None, "Non-existing device");
        
        assert_eq!(db.sub_devices.get(&(0x1022,0x15e4)), Some(&"Raven/Raven2/Renoir Sensor Fusion Hub".to_string()));
        assert_eq!(db.devices.get(&(0xfff0,0xfff0)), None, "Non-existing sub device");
    }

    #[test]
    fn class_codes() {
        let data = include_str!("/usr/share/hwdata/pci.ids");
        let mut class_codes = ClassCodes::new(data);
        let result = class_codes
            .find(|ClassCode { class, sub_class, prog_if }| {
                class.id == 0x09 && 
                sub_class.as_ref().map(|sc| sc.id) == Some(0x04) &&
                prog_if.as_ref().map(|pi| pi.id) == Some(0x10)
            })
            .unwrap();
        let sample = ClassCode {
            class: Class { id: 0x09, name: "Input device controller" },
            sub_class: Some(SubClass { id: 0x04, name: "Gameport controller" }),
            prog_if: Some(ProgIf { id: 0x10, name: "Extended" }),
        };
        assert_eq!(sample, result);
    }
}
