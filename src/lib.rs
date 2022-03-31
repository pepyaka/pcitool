//! This crate contains a library for portable access to PCI bus configuration registers.
//!
//!

use core::convert::TryFrom;



pub mod pciids;
pub mod access;
pub mod device;
pub mod view;


/// Accessory trait for uint conversion
trait FromBytes<'a> {
    type Data: TryFrom<&'a [u8]>;
    fn from_bytes(data: Self::Data) -> Self;
}
impl<'a> FromBytes<'a> for u8 {
    type Data = [u8; 1];
    fn from_bytes(data: Self::Data) -> Self { u8::from_ne_bytes(data) }
}
impl<'a> FromBytes<'a> for u16 {
    type Data = [u8; 2];
    fn from_bytes(data: Self::Data) -> Self { u16::from_ne_bytes(data) }
}
impl<'a> FromBytes<'a> for u32 {
    type Data = [u8; 4];
    fn from_bytes(data: Self::Data) -> Self { u32::from_ne_bytes(data) }
}
impl<'a> FromBytes<'a> for u64 {
    type Data = [u8; 8];
    fn from_bytes(data: Self::Data) -> Self { u64::from_ne_bytes(data) }
}
impl<'a> FromBytes<'a> for u128 {
    type Data = [u8; 16];
    fn from_bytes(data: Self::Data) -> Self { u128::from_ne_bytes(data) }
}
impl<'a, const N: usize> FromBytes<'a> for [u8; N] {
    type Data = [u8; N];
    fn from_bytes(data: Self::Data) -> Self { data }
}
