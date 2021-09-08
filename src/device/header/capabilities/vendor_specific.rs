//! Vendor Specific
//!
//! Allow device vendors to use the capability mechanism for vendor specific information. The
//! layout of the information is vendor specific, except that the byte immediately following the
//! “Next” pointer in the capability structure is defined to be a length field. 
//! An example vendor specific usage is a device that is configured in the final
//! manufacturing steps as either a 32-bit or 64-bit PCI agent and the Vendor Specific capability
//! structure tells the device driver which features the device supports. 


use core::convert::TryFrom;
use core::fmt::{self, Display, Formatter};
use core::cell::Cell;
use core::convert::TryInto;

use thiserror::Error;

use crate::{DisplayView, View};

/// Vendor Specific capability depends on Vendor ID and Device ID
#[derive(Debug, Default)]
pub struct VendorSpecific<'a> {
    pub vendor_id: u16,
    pub device_id: u16,
    /// Vendor capability data without data length, capability ID and Next pointer bytes 
    /// (pure data)
    data: &'a [u8],
    _view: Cell<View>,
}

#[derive(Error, Debug)]
pub enum VendorSpecificError {
    #[error("length getting problem")]
    UnknownLength,
    #[error("data slice getting problem")]
    UnknownData,
}

impl<'a> VendorSpecific<'a> {
    fn fmt_virtio(&self, f: &mut Formatter<'_>, verbose: u64) -> fmt::Result {
        let d = self.data;
        let type_ = d.get(0);
        let tname = match type_ {
            Some(1) => "CommonCfg",
            Some(2) => "Notify",
            Some(3) => "ISR",
            Some(4) => "DeviceCfg",
            _ => "<unknown>",
        };
        write!(f, "VirtIO: {}\n", tname)?;
        if verbose < 2 {
            return Ok(())
        }
        if let (Some(b), Some(o), Some(s)) = (
            d.get(1),
            d.get(5..9).and_then(|b| b.try_into().ok()).map(|b| i32::from_ne_bytes(b)),
            d.get(9..13).and_then(|b| b.try_into().ok()).map(|b| i32::from_ne_bytes(b)),
        ) {
            write!(f, "\t\tBAR={} offset={:08x} size={:08x}", b, o, s)?;
        }
        if let (true, Some(m)) = (type_ == Some(&2) && d.len() + 3 >= 20, d.get(13)) {
            write!(f, " multiplier={:08x}", m)?;
        }
        write!(f, "\n")
    }
}

impl<'a> TryFrom<&'a [u8]> for VendorSpecific<'a> {
    type Error = VendorSpecificError;
    fn try_from(slice: &'a [u8]) -> Result<Self, Self::Error> {
        let length = *slice.get(2).ok_or(VendorSpecificError::UnknownLength)? as usize;
        let data = slice.get(3..length).ok_or(VendorSpecificError::UnknownData)?;
        Ok(Self { data, ..Default::default() })
    }
}

///  Vendor Specific view depends on vendor_id and device_id
///  so it have own display() implementation
impl<'a> DisplayView<'a> for VendorSpecific<'a> {
    type View = &'a VendorSpecific<'a>;
    fn display(&'a self, view: View) -> Self::View {
        self._view.set(view);
        self
    }
}
impl<'a> Display for VendorSpecific<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self._view.take() {
            View::Basic => {
                Ok(())
            },
            View::Lspci(verbose) => {
                write!(f, "Vendor Specific Information: ")?;
                match (self.vendor_id, self.device_id) {
                    (0x1af4, 0x1000..=0x107f) => self.fmt_virtio(f, verbose),
                    _ => write!(f, "Len={:02x} <?>\n", self.data.len() + 3),
                }
            },
            View::Extended => {
                Ok(())
            }
        }
    }
}
