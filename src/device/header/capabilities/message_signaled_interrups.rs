//! Message Signaled Interrupts
//!
//! Message Signaled Interrupts (MSI) is an optional feature that enables a device function to
//! request service by writing a system-specified data value to a system-specified address (using a
//! PCI DWORD memory write transaction). System software initializes the message address and
//! message data (from here on referred to as the “vector”) during device configuration, allocating
//! one or more vectors to each MSI capable function.


use core::cell::Cell;
use core::fmt::{self, Display, Formatter};
use core::convert::TryFrom;
use core::array::TryFromSliceError;
use core::ops::Range;
use std::convert::TryInto;

use thiserror::Error;
use modular_bitfield::prelude::*;

use crate::{DisplayView, View, CheckBox};

/// Common Capability Structure for Message Address
#[bitfield(bits = 48)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct MsgAddrCommon {
    // capability_id: u8,
    // next_item_ptr: u8,
    ctrl_enable: bool,
    ctrl_multiple_message_capable: B3,
    ctrl_multiple_message_enable: B3,
    ctrl_is64bit: bool,
    ctrl_per_vector_masking_capable: bool,
    ctrl_reserved: B7,
    address: u32,
}

/// Capability Structure for 32-bit Message Address
#[bitfield(bits = 16)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct MsgAddr32Bit {
    data: u16,
}

/// Capability Structure for 64-bit Message Address
#[bitfield(bits = 48)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct MsgAddr64Bit {
    upper_address: u32,
    data: u16,
}

/// Capability Structure for 32-bit Message Address and Per-vector Masking
#[bitfield(bits = 96)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct MsgAddr32BitVecMsk {
    data: u16,
    #[allow(dead_code)]
    reserved: u16,
    mask_bits: u32,
    pending_bits: u32,
}

/// Capability Structure for 64-bit Message Address and Per-vector Masking
#[bitfield(bits = 128)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct MsgAddr64BitVecMsk {
    upper_address: u32,
    data: u16,
    #[allow(dead_code)]
    reserved: u16,
    mask_bits: u32,
    pending_bits: u32,
}

/// To request service, an MSI function writes the contents of the Message Data register to the
/// address specified by the contents of the Message Address register (and, optionally, the Message
/// Upper Address register for a 64-bit message address). A read of the address specified by the
/// contents of the Message Address register produces undefined results. 
#[derive(Default, Debug, PartialEq, Eq,)] 
pub struct MessageSignaledInterrups {
    pub message_control: MessageControl,
    pub message_address: MessageAddress,
    /// System-specified message data
    pub message_data: u16,
    /// For each Mask bit that is set, the function is prohibited from sending the associated
    /// message
    pub mask_bits: Option<u32>,
    /// For each Pending bit that is set, the function has a pending associated message
    pub pending_bits: Option<u32>,
    _view: Cell<View>,
}

/// Provides system software control over MSI.
#[derive(Default, Debug, PartialEq, Eq,)] 
pub struct MessageControl {
    pub enable: bool,
    pub multiple_message_capable: NumberOfVectors,
    pub multiple_message_enable: NumberOfVectors,
    pub per_vector_masking_capable: bool,
    pub reserved: u8,

}

/// System-specified message address
#[derive(Debug, PartialEq, Eq,)] 
pub enum MessageAddress {
    Dword(u32),
    Qword(u64),
}

/// The number of requested vectors must be aligned to a power of two (if a function requires three
/// vectors, it requests four by initializing this field to “010”).
#[derive(Clone, Copy, Debug, PartialEq, Eq,)] 
#[repr(u8)]
pub enum NumberOfVectors {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
    Reserved = u8::MAX,
}

/// Parsing errors
#[derive(Error, Debug)]
pub enum MessageSignaledInterrupsError {
    #[error("bytes range {0:?} is not available")]
    DataRange(Range<usize>),
    #[error("error while converting slice to array")]
    IntoArray(#[from] TryFromSliceError),
}

impl From<u8> for NumberOfVectors {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Four,
            3 => Self::Eight,
            4 => Self::Sixteen,
            5 => Self::ThirtyTwo,
            _ => Self::Reserved,
        }
    }
}

impl Default for NumberOfVectors {
    fn default() -> Self {
        Self::One
    }
}

impl Default for MessageAddress {
    fn default() -> Self {
        Self::Dword(Default::default())
    }
}

impl<'a> From<&'a MsgAddrCommon> for MessageControl {
    fn from(common: &'a MsgAddrCommon) -> Self {
       Self {
           enable: common.ctrl_enable(),
           multiple_message_capable: common.ctrl_multiple_message_capable().into(),
           multiple_message_enable: common.ctrl_multiple_message_enable().into(),
           per_vector_masking_capable: common.ctrl_per_vector_masking_capable(),
           reserved: common.ctrl_reserved(),
       }
    }
}

impl MessageSignaledInterrups {
    fn from_32bit(common: &MsgAddrCommon, ma: MsgAddr32Bit) -> Self {
        Self {
            message_control: common.into(),
            message_address: MessageAddress::Dword(common.address()),
            message_data: ma.data(),
            ..Default::default()
        }
    }
    fn from_64bit(common: &MsgAddrCommon, ma: MsgAddr64Bit) -> Self {
        let addr64 = (ma.upper_address() as u64) << 32 | common.address() as u64;
        Self {
            message_control: common.into(),
            message_address: MessageAddress::Qword(addr64),
            message_data: ma.data(),
            ..Default::default()
        }
    }
    fn from_32bit_vec_mck(common: &MsgAddrCommon, ma: MsgAddr32BitVecMsk) -> Self {
        Self {
            message_control: common.into(),
            message_address: MessageAddress::Dword(common.address()),
            message_data: ma.data(),
            mask_bits: Some(ma.mask_bits()),
            pending_bits: Some(ma.pending_bits()),
            ..Default::default()
        }
    }
    fn from_64bit_vec_msk(common: &MsgAddrCommon, ma: MsgAddr64BitVecMsk) -> Self {
        let addr64 = (ma.upper_address() as u64) << 32 | common.address() as u64;
        Self {
            message_control: common.into(),
            message_address: MessageAddress::Qword(addr64),
            message_data: ma.data(),
            mask_bits: Some(ma.mask_bits()),
            pending_bits: Some(ma.pending_bits()),
            ..Default::default()
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for MessageSignaledInterrups {
    type Error = MessageSignaledInterrupsError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let common = s2a(&data, 0x02..0x08).map(MsgAddrCommon::from_bytes)?;
        match (common.ctrl_is64bit(), common.ctrl_per_vector_masking_capable()) {
            (false, false) => s2a(&data, 0x08..0x0A)
                    .map(MsgAddr32Bit::from_bytes)
                    .map(|ma| MessageSignaledInterrups::from_32bit(&common, ma)),
            (true, false) => s2a(&data, 0x08..0x0E)
                    .map(MsgAddr64Bit::from_bytes)
                    .map(|ma| MessageSignaledInterrups::from_64bit(&common, ma)),
            (false, true) => s2a(&data, 0x08..0x14)
                    .map(MsgAddr32BitVecMsk::from_bytes)
                    .map(|ma| MessageSignaledInterrups::from_32bit_vec_mck(&common, ma)),
            (true, true) => s2a(&data, 0x08..0x18)
                    .map(MsgAddr64BitVecMsk::from_bytes)
                    .map(|ma| MessageSignaledInterrups::from_64bit_vec_msk(&common, ma)),
        }
    }
}
impl<'a> DisplayView<'a> for MessageSignaledInterrups {
    type View = &'a Self;
    fn display(&'a self, view: View) -> Self::View {
        self._view.set(view);
        self
    }
}
impl Display for MessageSignaledInterrups {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (ctrl, addr, data, mask, pend) = (
            &self.message_control, 
            &self.message_address, 
            &self.message_data, 
            &self.mask_bits, 
            &self.pending_bits
        );
        match self._view.take() {
            View::Basic => {
                Ok(())
            },
            View::Lspci(verbose) => {
                write!(f, "MSI: Enable{} Count={}/{} Maskable{} 64bit{}\n", 
                    CheckBox::lspci(ctrl.enable),
                    ctrl.multiple_message_enable as u8,
                    ctrl.multiple_message_capable as u8,
                    CheckBox::lspci(ctrl.per_vector_masking_capable),
                    CheckBox::lspci(matches!(addr, MessageAddress::Qword(_))),
                )?;
                if verbose < 2 {
                    return Ok(())
                }
                match addr {
                    MessageAddress::Dword(v) => {
                        write!(f, "\t\tAddress: {:08x}  Data: {:04x}\n", v, data)?;
                    },
                    MessageAddress::Qword(v) => {
                        write!(f, "\t\tAddress: {:016x}  Data: {:04x}\n", v, data)?;
                    },
                }
                if let (Some(m), Some(p)) = (mask, pend) {
                    write!(f, "\t\tMasking: {:08x}  Pending: {:08x}\n", m, p)?;
                }
                Ok(())
            },
            View::Extended => {
                Ok(())
            }
        }
    }
}


fn s2a<'a, const N: usize>(data: &'a [u8], range: Range<usize>) -> Result<[u8; N], MessageSignaledInterrupsError> {
    data.get(range.clone())
        .ok_or(MessageSignaledInterrupsError::DataRange(range))
        .and_then(|s| s.try_into().map_err(MessageSignaledInterrupsError::IntoArray))
}


#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;
    //use extfmt::{hexdump, AsHexdump};

    #[test]
    fn msg_addr() {
        let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/random/4k"));
        //println!("{}", hexdump!(data));
        let mac = MsgAddrCommon::from_bytes(data[0x2..0x8].try_into().unwrap());
        assert_eq!(true, mac.ctrl_enable());
        assert_eq!(1, mac.ctrl_multiple_message_capable());
        assert_eq!(6, mac.ctrl_multiple_message_enable());
        assert_eq!(false, mac.ctrl_is64bit());
        assert_eq!(false, mac.ctrl_per_vector_masking_capable());
        assert_eq!(66, mac.ctrl_reserved());
        assert_eq!(0x95f8e4dc, mac.address());
    }

    #[test]
    fn number_of_vectors() {
        for i in 0..6 {
            assert_eq!(1 << i, NumberOfVectors::from(i) as u8);
        }
    }

    #[test]
    fn message_address_32bit() {
        let mut data: [u8; 0x0A] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/random/4k"))
            [..0x0A]
            .try_into().unwrap();
        let control = 0b0_0000_0000u16.to_le_bytes();
        data[2] = control[0];
        data[3] = control[1];
        let result: MessageSignaledInterrups = data[..].try_into().unwrap();
        let sample = MessageSignaledInterrups {
            message_control: MessageControl {
                enable: false,
                multiple_message_capable: NumberOfVectors::One,
                multiple_message_enable: NumberOfVectors::One,
                per_vector_masking_capable: false,
                reserved: 0,
            },
            message_address: MessageAddress::Dword(0x95f8e4dc),
            message_data: 0xcb86,
            ..Default::default()
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn message_address_64bit() {
        let mut data: [u8; 0x0E] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/random/4k"))
            [..0x0E]
            .try_into().unwrap();
        let control = 0b0_1000_0000u16.to_le_bytes();
        data[2] = control[0];
        data[3] = control[1];
        let result: MessageSignaledInterrups = data[..].try_into().unwrap();
        let sample = MessageSignaledInterrups {
            message_control: MessageControl {
                enable: false,
                multiple_message_capable: NumberOfVectors::One,
                multiple_message_enable: NumberOfVectors::One,
                per_vector_masking_capable: false,
                reserved: 0,
            },
            message_address: MessageAddress::Qword(0x1A87CB8695F8E4DC),
            message_data: 0x5eb6,
            ..Default::default()
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn message_address_32bit_per_vector_masking() {
        let mut data: [u8; 0x14] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/random/4k"))
            [..0x14]
            .try_into().unwrap();
        let control = 0b1_0000_0000u16.to_le_bytes();
        data[2] = control[0];
        data[3] = control[1];
        let result: MessageSignaledInterrups = data[..].try_into().unwrap();
        let sample = MessageSignaledInterrups {
            message_control: MessageControl {
                enable: false,
                multiple_message_capable: NumberOfVectors::One,
                multiple_message_enable: NumberOfVectors::One,
                per_vector_masking_capable: true,
                reserved: 0,
            },
            message_address: MessageAddress::Dword(0x95f8e4dc),
            message_data: 0xcb86,
            mask_bits: Some(0x32735EB6),
            pending_bits: Some(0x6AE226D5),
            ..Default::default()
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn message_address_64bit_per_vector_masking() {
        let mut data: [u8; 0x18] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/random/4k"))
            [..0x18]
            .try_into().unwrap();
        let control = 0b1_1000_0000u16.to_le_bytes();
        data[2] = control[0];
        data[3] = control[1];
        let result: MessageSignaledInterrups = data[..].try_into().unwrap();
        let sample = MessageSignaledInterrups {
            message_control: MessageControl {
                enable: false,
                multiple_message_capable: NumberOfVectors::One,
                multiple_message_enable: NumberOfVectors::One,
                per_vector_masking_capable: true,
                reserved: 0,
            },
            message_address: MessageAddress::Qword(0x1A87CB8695F8E4DC),
            message_data: 0x5eb6,
            mask_bits: Some(0x6AE226D5),
            pending_bits: Some(0x5FFFEED8),
            ..Default::default()
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn display_lspci() {
        let data = &include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:2030/config"))
            [0x60..(0x60 + 0x18)];
        let result: MessageSignaledInterrups = data.try_into().unwrap();
        let sample = "\
            MSI: Enable+ Count=1/2 Maskable+ 64bit-\n\
            \t\tAddress: fee00038  Data: 0000\n\
            \t\tMasking: 00000002  Pending: 00000000\n\
        ";
        assert_eq!(sample, format!("{}", result.display(View::Lspci(3))));
    }
}
