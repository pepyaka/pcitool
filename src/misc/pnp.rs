/*!
# Plug and Play Resource Data Types

Plug and Play resource data fully describes all resource requirements of a Plug and Play ISA card as well as
resource programmability and interdependencies. Plug and Play resource data is supplied as a series of
“tagged” data structures.
*/

use heterob::{
    bit_numbering::Lsb,
    endianness::{Le, LeBytesTryInto},
    Seq, P2, P3, U8,
};

/// An iterator over Plug and Play resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlugAndPlayResource<'a> {
    data: &'a [u8],
}

impl<'a> PlugAndPlayResource<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for PlugAndPlayResource<'a> {
    type Item = Resource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (first, data) = self.data.split_first()?;
        let Lsb((large_item_name, is_large_item)) = P2::<_, 7, 1>(*first).into();
        let Lsb((small_item_len, small_item_name)) = P2::<u8, 3, 4>(large_item_name).into();
        let _: u8 = small_item_len;
        let (resource, tail) = if is_large_item {
            let Seq {
                head: large_item_len,
                tail,
            } = data.le_bytes_try_into().ok()?;
            if tail.len() < large_item_len as usize {
                return None;
            }
            let (data, tail) = tail.split_at(large_item_len as usize);
            let item = match large_item_name {
                0x01 => LargeItem::MemoryRangeDescriptor,
                0x02 => LargeItem::IdentifierStringAnsi(std::str::from_utf8(data).ok()?),
                0x03 => LargeItem::IdentifierStringUnicode(std::str::from_utf8(data).ok()?),
                0x04 => LargeItem::VendorDefined,
                0x05 => LargeItem::MemoryRangeDescriptor32bit,
                0x06 => LargeItem::FixedLocationMemoryRangeDescriptor32bit,
                0x10 => LargeItem::VitalProductDataRo(VitalProductDataRo::new(data)),
                0x11 => LargeItem::VitalProductDataRw(VitalProductDataRw::new(data)),
                v => LargeItem::Reserved(v),
            };
            (
                Resource::Large(Large {
                    item,
                    length: large_item_len,
                }),
                tail,
            )
        } else {
            if data.len() < small_item_len as usize {
                return None;
            }
            let (_data, tail) = data.split_at(small_item_len as usize);
            let item = match small_item_name {
                0x01 => SmallItem::PlugAndPlayVersionNumber,
                0x02 => SmallItem::LogicalDeviceId,
                0x03 => SmallItem::CompatibleDeviceId,
                0x04 => SmallItem::IrqFormat,
                0x05 => SmallItem::DmaFormat,
                0x06 => SmallItem::StartDependentFunction,
                0x07 => SmallItem::EndDependentFunction,
                0x08 => SmallItem::IoPortDescriptor,
                0x09 => SmallItem::FixedLocationIoPortDescriptor,
                0x0E => SmallItem::VendorDefined,
                0x0F => SmallItem::End,
                v => SmallItem::Reserved(v),
            };
            (
                Resource::Small(Small {
                    item,
                    length: small_item_len,
                }),
                tail,
            )
        };
        self.data = tail;
        Some(resource)
    }
}

/// To minimize the amount of storage needed on Plug and Play ISA cards two different
/// data types are supported. These are called small items and large items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource<'a> {
    Small(Small),
    Large(Large<'a>),
}

/// Small Resource Data Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Small {
    pub item: SmallItem,
    pub length: u8,
}

/// Small information items
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmallItem {
    /// Plug and Play version number
    PlugAndPlayVersionNumber,
    /// Logical device ID
    LogicalDeviceId,
    /// Compatible device ID
    CompatibleDeviceId,
    /// IRQ format
    IrqFormat,
    /// DMA format
    DmaFormat,
    /// Start dependent Function
    StartDependentFunction,
    /// End dependent Function
    EndDependentFunction,
    /// I/O port descriptor
    IoPortDescriptor,
    /// Fixed location I/O port descriptor
    FixedLocationIoPortDescriptor,
    /// Reserved
    Reserved(u8),
    /// Vendor defined
    VendorDefined,
    /// End tag
    End,
}

impl SmallItem {
    pub fn value(&self) -> u8 {
        match self {
            SmallItem::PlugAndPlayVersionNumber => 0x01,
            SmallItem::LogicalDeviceId => 0x02,
            SmallItem::CompatibleDeviceId => 0x03,
            SmallItem::IrqFormat => 0x04,
            SmallItem::DmaFormat => 0x05,
            SmallItem::StartDependentFunction => 0x06,
            SmallItem::EndDependentFunction => 0x07,
            SmallItem::IoPortDescriptor => 0x08,
            SmallItem::FixedLocationIoPortDescriptor => 0x09,
            SmallItem::VendorDefined => 0x0E,
            SmallItem::End => 0x0F,
            SmallItem::Reserved(v) => *v,
        }
    }
}

/// Large Resource Data Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Large<'a> {
    pub item: LargeItem<'a>,
    pub length: u16,
}

/// Large information items
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LargeItem<'a> {
    /// Memory range descriptor
    MemoryRangeDescriptor,
    /// Identifier string (ANSI)
    IdentifierStringAnsi(&'a str),
    /// Identifier string (Unicode)
    IdentifierStringUnicode(&'a str),
    /// Vendor defined
    VendorDefined,
    /// 32-bit memory range descriptor
    MemoryRangeDescriptor32bit,
    /// 32-bit fixed location memory range descriptor
    FixedLocationMemoryRangeDescriptor32bit,
    /// Read-only Vital Product Data
    VitalProductDataRo(VitalProductDataRo<'a>),
    /// Read-write Vital Product Data
    VitalProductDataRw(VitalProductDataRw<'a>),
    /// Reserved
    Reserved(u8),
}

impl<'a> LargeItem<'a> {
    pub fn value(&self) -> u8 {
        match self {
            LargeItem::MemoryRangeDescriptor => 0x01,
            LargeItem::IdentifierStringAnsi(_) => 0x02,
            LargeItem::IdentifierStringUnicode(_) => 0x03,
            LargeItem::VendorDefined => 0x04,
            LargeItem::MemoryRangeDescriptor32bit => 0x05,
            LargeItem::FixedLocationMemoryRangeDescriptor32bit => 0x06,
            LargeItem::VitalProductDataRo(_) => 0x10,
            LargeItem::VitalProductDataRw(_) => 0x11,
            LargeItem::Reserved(v) => *v,
        }
    }
}

/// An iterator over read-only VPD resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VitalProductDataRo<'a> {
    data: &'a [u8],
}

impl<'a> VitalProductDataRo<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for VitalProductDataRo<'a> {
    type Item = VpdRoResource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Seq {
            head: Le((U8(k0), U8(k1), U8(len))),
            tail,
        } = P3(self.data).try_into().ok()?;
        if tail.len() < len {
            return None;
        }
        let (data, tail) = tail.split_at(len);
        self.data = tail;
        let result = match (k0, k1) {
            ('P', 'N') => VpdRoResource::PartNumber(std::str::from_utf8(data).ok()?),
            ('E', 'C') => VpdRoResource::EngineeringChange(std::str::from_utf8(data).ok()?),
            ('F', 'G') => VpdRoResource::FabricGeography(std::str::from_utf8(data).ok()?),
            ('L', 'C') => VpdRoResource::Location(std::str::from_utf8(data).ok()?),
            ('M', 'N') => VpdRoResource::ManufactureId(std::str::from_utf8(data).ok()?),
            ('P', 'G') => VpdRoResource::PciGeography(std::str::from_utf8(data).ok()?),
            ('S', 'N') => VpdRoResource::SerialNumber(std::str::from_utf8(data).ok()?),
            ('V', x) => VpdRoResource::VendorSpecific(x, std::str::from_utf8(data).ok()?),
            ('C', 'P') => {
                let Seq {
                    head: Le((cap_id, bar_index, bar_offset)),
                    ..
                } = P3(data).try_into().ok()?;
                VpdRoResource::ExtendedCapability {
                    cap_id,
                    bar_index,
                    bar_offset,
                }
            }
            ('R', 'V') => {
                let Seq {
                    head: checksum,
                    tail: reserved,
                } = data.le_bytes_try_into().ok()?;
                VpdRoResource::ChecksumAndReserved { checksum, reserved }
            }
            (k0, k1) => VpdRoResource::Unknown {
                k0,
                k1,
                len: len as u8,
                data,
            },
        };
        Some(result)
    }
}
/// VPD read-only Fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpdRoResource<'a> {
    /// PN Add-in Card Part Number
    PartNumber(&'a str),
    /// EC Engineering Change Level of the Add-in Card
    EngineeringChange(&'a str),
    /// FG Fabric Geography
    FabricGeography(&'a str),
    /// LC Location
    Location(&'a str),
    /// MN Manufacture ID
    ManufactureId(&'a str),
    /// PG PCI Geography
    PciGeography(&'a str),
    /// SN Serial Number
    SerialNumber(&'a str),
    /// Vx Vendor Specific
    VendorSpecific(char, &'a str),
    /// CP Extended Capability
    ExtendedCapability {
        cap_id: u8,
        bar_index: u8,
        bar_offset: u16,
    },
    /// RV Checksum and Reserved
    ChecksumAndReserved { checksum: u8, reserved: &'a [u8] },
    /// Unknown
    Unknown {
        k0: char,
        k1: char,
        len: u8,
        data: &'a [u8],
    },
}

/// An iterator over read-only VPD resources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VitalProductDataRw<'a> {
    data: &'a [u8],
}

impl<'a> VitalProductDataRw<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for VitalProductDataRw<'a> {
    type Item = VpdRwResource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Seq {
            head: Le((U8(k0), U8(k1), U8(len))),
            tail,
        } = P3(self.data).try_into().ok()?;
        if tail.len() < len {
            return None;
        }
        let (data, tail) = tail.split_at(len);
        self.data = tail;
        let result = match (k0, k1) {
            ('V', x) => VpdRwResource::VendorSpecific(x, std::str::from_utf8(data).ok()?),
            ('Y', 'A') => VpdRwResource::AssetTagIdentifier(std::str::from_utf8(data).ok()?),
            ('Y', x) => VpdRwResource::SystemSpecific(x, data),
            _ => VpdRwResource::RemainingRwArea(data),
        };
        Some(result)
    }
}

/// VPD read-write fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpdRwResource<'a> {
    /// Vx Vendor Specific
    VendorSpecific(char, &'a str),
    /// Yx System Specific
    SystemSpecific(char, &'a [u8]),
    /// YA Asset Tag Identifier
    AssetTagIdentifier(&'a str),
    /// RW Remaining Read/Write Area
    RemainingRwArea(&'a [u8]),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn vital_product_data_ro() {
        let data = [
            b"PN",
            [0x08].as_slice(),
            b"6181682A",
            b"EC",
            [0x0A].as_slice(),
            b"4950262536",
            b"SN",
            [0x08].as_slice(),
            b"00000194",
            b"MN",
            [0x04].as_slice(),
            b"1037",
            b"RV",
            [0x2C].as_slice(),
            [0xFF].as_slice(), // Checksum
            [0x00; 128 - 85].as_slice(),
        ]
        .concat();

        let sample_vpd_r = vec![
            VpdRoResource::PartNumber("6181682A"),
            VpdRoResource::EngineeringChange("4950262536"),
            VpdRoResource::SerialNumber("00000194"),
            VpdRoResource::ManufactureId("1037"),
            VpdRoResource::ChecksumAndReserved {
                checksum: 0xFF,
                reserved: &[0x00; 0x2c - 1],
            },
        ];
        let result_vpd_r = VitalProductDataRo::new(&data);
        assert_eq!(sample_vpd_r, result_vpd_r.collect::<Vec<_>>());
    }

    #[test]
    fn vital_product_data_rw() {
        let data = [
            b"V1",
            [0x05].as_slice(),
            b"65A01",
            b"Y1",
            [0x0D].as_slice(),
            b"Error Code 26",
            b"RW",
            [0x61].as_slice(),
            [0x00; 255 - 158].as_slice(),
        ]
        .concat();

        let sample_vpd_w = vec![
            VpdRwResource::VendorSpecific('1', "65A01"),
            VpdRwResource::SystemSpecific('1', b"Error Code 26"),
            VpdRwResource::RemainingRwArea(&[0; 0x61]),
        ];
        let result_vpd_w = VitalProductDataRw::new(&data);
        assert_eq!(sample_vpd_w, result_vpd_w.clone().collect::<Vec<_>>());
    }

    #[test]
    fn plug_and_play_resource() {
        let data = [
            [0x82].as_slice(),
            &0x0021u16.to_le_bytes(),
            b"ABCD Super-Fast Widget Controller",
            [0x90].as_slice(),
            &0x0059u16.to_le_bytes(),
            b"PN",
            [0x08].as_slice(),
            b"6181682A",
            b"EC",
            [0x0A].as_slice(),
            b"4950262536",
            b"SN",
            [0x08].as_slice(),
            b"00000194",
            b"MN",
            [0x04].as_slice(),
            b"1037",
            b"RV",
            [0x2C].as_slice(),
            [0xFF].as_slice(), // Checksum
            [0x00; 128 - 85].as_slice(),
            [0x91].as_slice(),
            &0x007Cu16.to_le_bytes(),
            b"V1",
            [0x05].as_slice(),
            b"65A01",
            b"Y1",
            [0x0D].as_slice(),
            b"Error Code 26",
            b"RW",
            [0x61].as_slice(),
            [0x00; 255 - 158].as_slice(),
            [0x78].as_slice(),
        ]
        .concat();

        let sample = vec![
            Resource::Large(Large {
                item: LargeItem::IdentifierStringAnsi("ABCD Super-Fast Widget Controller"),
                length: 0x0021,
            }),
            Resource::Large(Large {
                item: LargeItem::VitalProductDataRo(VitalProductDataRo::new(&data[39..128])),
                length: 0x0059,
            }),
            Resource::Large(Large {
                item: LargeItem::VitalProductDataRw(VitalProductDataRw::new(&data[131..255])),
                length: 0x007C,
            }),
            Resource::Small(Small {
                item: SmallItem::End,
                length: 0x00,
            }),
        ];
        let result = PlugAndPlayResource::new(&data);
        assert_eq!(sample, result.clone().collect::<Vec<_>>(), "{:x?}", result);
    }
}
