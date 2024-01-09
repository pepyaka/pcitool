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
                (67 /* C */, 67 /* C */) => {
                    writeln!(f, "[CC] CCIN: {}", String::from_utf8_lossy(data))
                }
                (70 /* F */, 67 /* C */) => {
                    writeln!(f, "[FC] Feature code: {}", String::from_utf8_lossy(data))
                }
                (70 /* F */, 78 /* N */) => {
                    writeln!(f, "[FN] FRU: {}", String::from_utf8_lossy(data))
                }
                (78 /* N */, 65 /* A */) => {
                    writeln!(f, "[NA] Network address: {}", String::from_utf8_lossy(data))
                }
                (82 /* R */, 77 /* M */) => writeln!(
                    f,
                    "[RM] Firmware version: {}",
                    String::from_utf8_lossy(data)
                ),
                (90 /* Z */, k1) => writeln!(
                    f,
                    "[Z{}] Product specific: {}",
                    k1 as char,
                    String::from_utf8_lossy(data)
                ),
                (k0, k1) => writeln!(
                    f,
                    "[{}] Unknown: {}",
                    VpdBytes(&[k0, k1]),
                    data.iter()
                        .map(|b| format!("{:02x} ", b))
                        .collect::<String>()
                        .trim()
                ),
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
        write!(f, "{}", VpdBytes(self.0.as_bytes()))
    }
}

struct VpdBytes<'a>(&'a [u8]);

impl<'a> fmt::Display for VpdBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.0.is_empty() {
            let mut chars = self.0.iter().peekable();
            while let Some(ch) = chars.next() {
                match ch {
                    0 if chars.peek().is_none() => (),
                    92 => write!(f, "\\\\")?, // '\\' - symbol
                    ch @ (0..=31 | 127) => write!(f, "\\x{:02x}", ch)?,
                    _ => write!(f, "{}", *ch as char)?,
                }
            }
        }
        Ok(())
    }
}
