use std::str::{FromStr, Lines};

use super::{CcKey, Modalias, VdsKey};

#[derive(Debug, Default)]
pub struct HwDb;

pub struct VendorModel<'a> {
    lines: Lines<'a>,
}

impl<'a> VendorModel<'a> {
    pub const PATH: &'static str = "/lib/udev/hwdb.d/20-pci-vendor-model.hwdb";
    pub fn new(s: &'a str) -> Self {
        Self { lines: s.lines() }
    }
}

impl<'a> Iterator for VendorModel<'a> {
    type Item = VendorModelEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(modalias) = Modalias::from_str(line) {
                if let Some(property) = self.lines.next().and_then(|s| s.parse().ok()) {
                    return Some(VendorModelEntry { modalias, property });
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VendorModelEntry {
    modalias: Modalias,
    property: VendorModelProperty,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct VendorModelPropertyError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VendorModelProperty {
    Vendor(String),
    Model(String),
}

impl FromStr for VendorModelProperty {
    type Err = VendorModelPropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vendor = s
            .strip_prefix(" ID_VENDOR_FROM_DATABASE=")
            .map(|s| VendorModelProperty::Vendor(s.into()));
        let model = s
            .strip_prefix(" ID_MODEL_FROM_DATABASE=")
            .map(|s| VendorModelProperty::Model(s.into()));
        vendor.or(model).ok_or(VendorModelPropertyError)
    }
}

impl FromIterator<VendorModelEntry> for super::VendorDeviceSubsystem {
    fn from_iter<T: IntoIterator<Item = VendorModelEntry>>(iter: T) -> Self {
        let vds = iter
            .into_iter()
            .filter_map(|entry| {
                let vds_key = VdsKey::from(entry.modalias);
                match (vds_key, entry.property) {
                    (k @ VdsKey::Vendor(..), VendorModelProperty::Vendor(v)) => Some((k, v)),
                    (k @ VdsKey::Device(..), VendorModelProperty::Model(v)) => Some((k, v)),
                    (k @ VdsKey::Subsystem(..), VendorModelProperty::Model(v)) => Some((k, v)),
                    _ => None,
                }
            })
            .collect();
        Self(vds)
    }
}

pub struct Classes<'a> {
    lines: Lines<'a>,
}

impl<'a> Classes<'a> {
    pub const PATH: &'static str = "/lib/udev/hwdb.d/20-pci-classes.hwdb";
    pub fn new(s: &'a str) -> Self {
        Self { lines: s.lines() }
    }
}

impl<'a> Iterator for Classes<'a> {
    type Item = ClassesEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.lines.next() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(modalias) = Modalias::from_str(line) {
                if let Some(property) = self.lines.next().and_then(|s| s.parse().ok()) {
                    return Some(ClassesEntry { modalias, property });
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassesEntry {
    modalias: Modalias,
    property: ClassesProperty,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ClassesPropertyError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassesProperty {
    Class(String),
    Subclass(String),
    Interface(String),
}

impl FromStr for ClassesProperty {
    type Err = ClassesPropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let class = s
            .strip_prefix(" ID_PCI_CLASS_FROM_DATABASE=")
            .map(|s| ClassesProperty::Class(s.into()));
        let subclass = s
            .strip_prefix(" ID_PCI_SUBCLASS_FROM_DATABASE=")
            .map(|s| ClassesProperty::Subclass(s.into()));
        let interface = s
            .strip_prefix(" ID_PCI_INTERFACE_FROM_DATABASE=")
            .map(|s| ClassesProperty::Interface(s.into()));
        class.or(subclass).or(interface).ok_or(ClassesPropertyError)
    }
}

impl FromIterator<ClassesEntry> for super::ClassCode {
    fn from_iter<T: IntoIterator<Item = ClassesEntry>>(iter: T) -> Self {
        let vds = iter
            .into_iter()
            .filter_map(|entry| {
                let cc_key = CcKey::from(entry.modalias);
                match (cc_key, entry.property) {
                    (k @ CcKey::Class(..), ClassesProperty::Class(v)) => Some((k, v)),
                    (k @ CcKey::Subclass(..), ClassesProperty::Subclass(v)) => Some((k, v)),
                    (k @ CcKey::ProgIf(..), ClassesProperty::Interface(v)) => Some((k, v)),
                    _ => None,
                }
            })
            .collect();
        Self(vds)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::names::{ClassCode, VendorDeviceSubsystem};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_valid_vendor_model() {
        let data = [
            " ID_VENDOR_FROM_DATABASE=",
            " ID_MODEL_FROM_DATABASE=",
            " ID_VENDOR_FROM_DATABASE=Diablo Technologies",
            " ID_MODEL_FROM_DATABASE=AIC-7850T/7856T [AVA-2902/4/6 / AHA-2910]",
        ];
        let result = data.map(|s| s.parse());
        let sample = [
            Ok(VendorModelProperty::Vendor("".into())),
            Ok(VendorModelProperty::Model("".into())),
            Ok(VendorModelProperty::Vendor("Diablo Technologies".into())),
            Ok(VendorModelProperty::Model(
                "AIC-7850T/7856T [AVA-2902/4/6 / AHA-2910]".into(),
            )),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn parse_invalid_vendor_model_modalias() {
        let data = [
            "",
            "\n",
            "ID_MODEL_FROM_DATABASE=A",
            " ID_VENDOR_FROM_DATABASE",
        ];
        let result = data.map(VendorModelProperty::from_str);
        let sample = [
            Err(VendorModelPropertyError),
            Err(VendorModelPropertyError),
            Err(VendorModelPropertyError),
            Err(VendorModelPropertyError),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn vendor_model() {
        let data = [
            "          # Comment",
            "                 ",
            "",
            "pci:v00001002d0000154Csv00001462sd00007C28*",
            " ID_MODEL_FROM_DATABASE=Kryptos [Radeon RX 350] (MS-7C28 Motherboard)",
            "",
            "pci:v00001002d0000154E*",
            " ID_VENDOR_FROM_DATABASE=Garfield",
        ]
        .join("\n");
        let vm = VendorModel::new(&data);
        let result: Vec<VendorModelEntry> = vm.collect();
        let sample = vec![
            VendorModelEntry {
                modalias: Modalias {
                    vendor: Some(0x1002),
                    device: Some(0x154c),
                    sub_vendor: Some(0x1462),
                    sub_device: Some(0x7c28),
                    base_class: None,
                    sub_class: None,
                    interface: None,
                },
                property: VendorModelProperty::Model(
                    "Kryptos [Radeon RX 350] (MS-7C28 Motherboard)".into(),
                ),
            },
            VendorModelEntry {
                modalias: Modalias {
                    vendor: Some(0x1002),
                    device: Some(0x154e),
                    sub_vendor: None,
                    sub_device: None,
                    base_class: None,
                    sub_class: None,
                    interface: None,
                },
                property: VendorModelProperty::Vendor("Garfield".into()),
            },
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn vds_from_iterator() {
        let data = [
            "          # Comment",
            "                 ",
            "",
            "pci:v00001002d0000154Csv00001462sd00007C28*",
            " ID_MODEL_FROM_DATABASE=Kryptos [Radeon RX 350]",
            "",
            "pci:v00001002*",
            " ID_VENDOR_FROM_DATABASE=SuperVendor",
        ]
        .join("\n");
        let vm = VendorModel::new(&data);
        let result: VendorDeviceSubsystem =
            vm.collect::<Vec<VendorModelEntry>>().into_iter().collect();
        let vds = [
            (
                VdsKey::Subsystem(0x1002, 0x154c, 0x1462, 0x7c28),
                "Kryptos [Radeon RX 350]".into(),
            ),
            (VdsKey::Vendor(0x1002), "SuperVendor".into()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert_eq!(VendorDeviceSubsystem(vds), result);
    }

    #[test]
    fn parse_valid_classes() {
        let data = [
            " ID_PCI_CLASS_FROM_DATABASE=",
            " ID_PCI_SUBCLASS_FROM_DATABASE=",
            " ID_PCI_INTERFACE_FROM_DATABASE=",
            " ID_PCI_CLASS_FROM_DATABASE=Unclassified device",
            " ID_PCI_SUBCLASS_FROM_DATABASE=IDE interface",
            " ID_PCI_INTERFACE_FROM_DATABASE=NVM Express",
        ];
        let result = data.map(|s| s.parse());
        let sample = [
            Ok(ClassesProperty::Class("".into())),
            Ok(ClassesProperty::Subclass("".into())),
            Ok(ClassesProperty::Interface("".into())),
            Ok(ClassesProperty::Class("Unclassified device".into())),
            Ok(ClassesProperty::Subclass("IDE interface".into())),
            Ok(ClassesProperty::Interface("NVM Express".into())),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn parse_invalid_classes_modalias() {
        let data = [
            "",
            "\n",
            "ID_PCI_INTERFACE_FROM_DATABASE=A",
            " ID_PCI_INTERFACE_FROM_DATABASE",
        ];
        let result = data.map(ClassesProperty::from_str);
        let sample = [
            Err(ClassesPropertyError),
            Err(ClassesPropertyError),
            Err(ClassesPropertyError),
            Err(ClassesPropertyError),
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn classes() {
        let data = [
            "# Comment",
            "            ",
            "",
            "pci:v*d*sv*sd*bc08*",
            " ID_PCI_CLASS_FROM_DATABASE=Generic system peripheral",
            "",
            "pci:v*d*sv*sd*bc08sc00*",
            " ID_PCI_SUBCLASS_FROM_DATABASE=PIC",
            "",
            "pci:v*d*sv*sd*bc08sc00i00*",
            " ID_PCI_INTERFACE_FROM_DATABASE=8259",
        ]
        .join("\n");
        let vm = Classes::new(&data);
        let result: Vec<ClassesEntry> = vm.collect();
        let sample = vec![
            ClassesEntry {
                modalias: Modalias {
                    vendor: None,
                    device: None,
                    sub_vendor: None,
                    sub_device: None,
                    base_class: Some(0x08),
                    sub_class: None,
                    interface: None,
                },
                property: ClassesProperty::Class("Generic system peripheral".into()),
            },
            ClassesEntry {
                modalias: Modalias {
                    vendor: None,
                    device: None,
                    sub_vendor: None,
                    sub_device: None,
                    base_class: Some(0x08),
                    sub_class: Some(0x00),
                    interface: None,
                },
                property: ClassesProperty::Subclass("PIC".into()),
            },
            ClassesEntry {
                modalias: Modalias {
                    vendor: None,
                    device: None,
                    sub_vendor: None,
                    sub_device: None,
                    base_class: Some(0x08),
                    sub_class: Some(0x00),
                    interface: Some(0x00),
                },
                property: ClassesProperty::Interface("8259".into()),
            },
        ];
        assert_eq!(sample, result);
    }

    #[test]
    fn cc_from_iterator() {
        let data = [
            "# Comment",
            "            ",
            "",
            "pci:v*d*sv*sd*bc08*",
            " ID_PCI_CLASS_FROM_DATABASE=Generic system peripheral",
            "",
            "pci:v*d*sv*sd*bc08sc00*",
            " ID_PCI_SUBCLASS_FROM_DATABASE=PIC",
            "",
            "pci:v*d*sv*sd*bc08sc00i00*",
            " ID_PCI_INTERFACE_FROM_DATABASE=8259",
        ]
        .join("\n");
        let vm = Classes::new(&data);
        let result: ClassCode = vm.collect::<Vec<ClassesEntry>>().into_iter().collect();
        let cc = [
            (CcKey::Class(0x08), "Generic system peripheral".into()),
            (CcKey::Subclass(0x08, 0x00), "PIC".into()),
            (CcKey::ProgIf(0x08, 0x00, 0x00), "8259".into()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert_eq!(ClassCode(cc), result);
    }
}
