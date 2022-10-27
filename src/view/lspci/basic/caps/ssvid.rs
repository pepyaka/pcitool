use std::fmt;

use pcics::capabilities::BridgeSubsystemVendorId;

use crate::names::VendorDeviceSubsystem;

use super::View;

pub(super) struct ViewArgs<'a> {
    pub(super) as_numbers: usize,
    pub(super) vds: &'a VendorDeviceSubsystem,
}

impl<'a> fmt::Display for View<&'a BridgeSubsystemVendorId, ViewArgs<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &BridgeSubsystemVendorId {
            subsystem_vendor_id: vendor_id,
            subsystem_id: device_id,
            ..
        } = self.data;
        let ViewArgs { as_numbers, vds } = self.args;
        let pciids_vendor = vds.lookup(vendor_id, None, None);
        let pciids_device = vds.lookup(vendor_id, device_id, None);
        write!(f, "Subsystem:")?;
        match (as_numbers, pciids_vendor, pciids_device) {
            (0, Some(v), Some(d)) => write!(f, " {} {}", v, d),
            (0, Some(v), _) => write!(f, " {} Device {:04x}", v, device_id),
            (0, _, _) => write!(f, " Device {:04x}", device_id),
            (1, _, _) => write!(f, " {:04x}:{:04x}", vendor_id, device_id),
            (_, Some(v), Some(d)) => {
                write!(f, " {} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id)
            }
            (_, Some(v), _) => write!(f, " {} Device [{:04x}:{:04x}]", v, vendor_id, device_id),
            _ => write!(f, " Device [{:04x}:{:04x}]", vendor_id, device_id),
        }
    }
}
