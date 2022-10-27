use core::fmt;

use pcics::extended_capabilities::{
    designated_vendor_specific_extended_capability::{
        compute_express_link::pcie_dvsec_for_cxl_device::{
            CacheCleanEviction, CxlStatus, PcieDvsecForCxlDevice,
        },
        ComputeExpressLink, DvsecError, DvsecType,
    },
    DesignatedVendorSpecificExtendedCapability as Dvsec,
};

use super::{Flag, Verbose, Simple};

impl<'a> fmt::Display for Verbose<&'a Dvsec<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let Dvsec {
                    dvsec_vendor_id,
                    dvsec_revision,
                    dvsec_length,
                    dvsec_id,
                    dvsec_type,
        } = self.data;
        let verbose = self.verbose;
        write!(
            f,
            "Designated Vendor-Specific: Vendor={:04x} ID={:04x} Rev={} Len={}",
            dvsec_vendor_id, dvsec_id, dvsec_revision, dvsec_length,
        )?;
        match dvsec_type {
            DvsecType::ComputeExpressLink(ComputeExpressLink::PcieDvsecForCxlDevice(
                PcieDvsecForCxlDevice {
                    cxl_capability,
                    cxl_control,
                    cxl_status: CxlStatus { viral_status },
                    ..
                },
            )) => {
                writeln!(f, ": CXL")?;
                if verbose < 2 {
                    return Ok(());
                }
                writeln!(
                    f,
                    "\t\tCXLCap:\tCache{} IO{} Mem{} Mem HW Init{} HDMCount {} Viral{}",
                    Flag(cxl_capability.cache_capable),
                    Flag(cxl_capability.io_capable),
                    Flag(cxl_capability.mem_capable),
                    Flag(cxl_capability.mem_hwinit_mode),
                    cxl_capability.hdm_count as u8,
                    Flag(cxl_capability.viral_capable),
                )?;
                writeln!(f, "\t\tCXLCtl:\tCache{} IO{} Mem{} Cache SF Cov {} Cache SF Gran {} Cache Clean{} Viral{}", 
                    Flag(cxl_control.cache_enable),
                    Flag(cxl_control.io_enable),
                    Flag(cxl_control.mem_enable),
                    Into::<u8>::into(cxl_control.cache_sf_coverage),
                    cxl_control.cache_sf_granularity as u8,
                    Flag(cxl_control.cache_clean_eviction == CacheCleanEviction::NotNeeded),
                    Flag(cxl_control.viral_enable),
                )?;
                writeln!(f, "\t\tCXLSta:\tViral{}", Flag(*viral_status))
            }
            _ => writeln!(f, " <?>"),
        }
    }
}

impl<'a> fmt::Display for Simple<&'a DvsecError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            DvsecError::VendorSpecificRegisters {
                dvsec_vendor_id,
                dvsec_revision,
                dvsec_length,
                dvsec_id,
                ..
            } => writeln!(
                f,
                "Designated Vendor-Specific: Vendor={:04x} ID={:04x} Rev={} Len={} <?>",
                dvsec_vendor_id, dvsec_id, dvsec_revision, dvsec_length,
            ),
            _ => writeln!(f, "Designated Vendor-Specific: <unreadable>"),
        }
    }
}
