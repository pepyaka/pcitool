use std::fmt;

use pcics::capabilities::VitalProductData;

use crate::misc::pnp::{
    Large, LargeItem, PlugAndPlayResource, Resource, Small, SmallItem, VpdRoResource, VpdRwResource,
};

use super::{Simple, View};

pub(super) struct ViewArgs<'a> {
    pub(super) verbose: usize,
    pub(super) pnp: Option<PlugAndPlayResource<'a>>,
}

impl<'a> fmt::Display for View<&'a VitalProductData, ViewArgs<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let VitalProductData { .. } = self.data;
        let ViewArgs { verbose, ref pnp } = self.args;
        writeln!(f, "Vital Product Data")?;
        if verbose < 2 {
            return Ok(());
        }
        // TODO: Iterate through all VPD addresses
        if let Some(vpd) = pnp.clone() {
            for resource in vpd {
                match resource {
                    Resource::Small(Small {
                        item: SmallItem::End,
                        ..
                    }) => writeln!(f, "\t\tEnd")?,
                    Resource::Small(Small { item, .. }) => writeln!(
                        f,
                        "\t\tUnknown small resource type {:02x}, will not decode more.",
                        item.value()
                    )?,
                    Resource::Large(Large {
                        item: LargeItem::IdentifierStringAnsi(s),
                        ..
                    }) => writeln!(f, "\t\tProduct Name: {}", VpdStr(s))?,
                    Resource::Large(Large {
                        item: LargeItem::VitalProductDataRo(vpd_r),
                        ..
                    }) => {
                        writeln!(f, "\t\tRead-only fields:")?;
                        for field in vpd_r {
                            write!(f, "\t\t\t{}", Simple(field))?;
                        }
                    }
                    Resource::Large(Large {
                        item: LargeItem::VitalProductDataRw(vpd_w),
                        ..
                    }) => {
                        writeln!(f, "\t\tRead/write fields:")?;
                        for field in vpd_w {
                            write!(f, "\t\t\t{}", Simple(field))?;
                        }
                    }
                    Resource::Large(Large { item, .. }) => writeln!(
                        f,
                        "\t\tUnknown large resource type {:02x}, will not decode more.",
                        item.value()
                    )?,
                }
            }
        } else {
            writeln!(f, "\t\tNot readable")?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Simple<VpdRoResource<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VpdRoResource::*;
        match self.0 {
            PartNumber(s) => writeln!(f, "[PN] Part number: {}", VpdStr(s)),
            EngineeringChange(s) => writeln!(f, "[EC] Engineering changes: {}", VpdStr(s)),
            FabricGeography(s) => writeln!(f, "[FG] Fabric geography: {}", VpdStr(s)),
            Location(s) => writeln!(f, "[LC] Location: {}", VpdStr(s)),
            ManufactureId(s) => writeln!(f, "[MN] Manufacture ID: {}", VpdStr(s)),
            PciGeography(s) => writeln!(f, ": {}", VpdStr(s)),
            SerialNumber(s) => writeln!(f, "[SN] Serial number: {}", VpdStr(s)),
            VendorSpecific(k1, s) => writeln!(f, "[V{}] Vendor specific: {}", k1, VpdStr(s)),
            ExtendedCapability { cap_id, .. } => writeln!(f, "Extended capability: {:x}", cap_id),
            ChecksumAndReserved { checksum, reserved } => writeln!(
                f,
                "[RV] Reserved: checksum {}, {} byte(s) reserved",
                if checksum != 0 { "good" } else { "bad" },
                reserved.len()
            ),
            Unknown { k0, k1, data, .. } => match (k0, k1) {
                /* Non-standard extensions */
                ('C', 'C') => writeln!(f, "[CC] CCIN: {}", String::from_utf8_lossy(data)),
                ('F', 'C') => writeln!(f, "[FC] Feature code: {}", String::from_utf8_lossy(data)),
                ('F', 'N') => writeln!(f, "[FN] FRU: {}", String::from_utf8_lossy(data)),
                ('N', 'A') => {
                    writeln!(f, "[NA] Network address: {}", String::from_utf8_lossy(data))
                }
                ('R', 'M') => writeln!(
                    f,
                    "[RM] Firmware version: {}",
                    String::from_utf8_lossy(data)
                ),
                ('Z', k1) => writeln!(
                    f,
                    "[Z{}] Product specific: {}",
                    k1,
                    String::from_utf8_lossy(data)
                ),
                (k0, k1) => writeln!(f, "[{}{}] Unknown: {:?}", k0, k1, data),
            },
        }
    }
}

impl<'a> fmt::Display for Simple<VpdRwResource<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VpdRwResource::*;
        match self.0 {
            VendorSpecific(k1, s) => writeln!(f, "[V{}] Vendor specific: {}", k1, VpdStr(s)),
            SystemSpecific(k1, s) => writeln!(
                f,
                "[Y{}] System specific: {}",
                k1,
                String::from_utf8_lossy(s)
            ),
            AssetTagIdentifier(s) => writeln!(f, "[YA] Asset tag: {}", VpdStr(s)),
            RemainingRwArea(d) => writeln!(f, "{} byte(s) free", d.len()),
        }
    }
}

struct VpdStr<'a>(&'a str);

impl<'a> fmt::Display for VpdStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            for ch in self.0.chars() {
                match ch as u8 {
                    0 => (),
                    92 => write!(f, "\\\\")?,
                    ch @ (0..=31 | 127) => write!(f, "\\x{:02x}", ch)?,
                    _ => write!(f, "{}", ch)?,
                }
            }
        }
        Ok(())
    }
}
