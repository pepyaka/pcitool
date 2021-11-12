//! Base Address Registers
//!
//! Base Address Registers (or BARs) can be used to hold memory addresses used by the device, or
//! offsets for port addresses. Typically, memory address BARs need to be located in physical ram
//! while I/O space BARs can reside at any memory address (even beyond physical memory). To
//! distinguish between them, you can check the value of the lowest bit.

use std::{slice,iter};

use serde::{Deserialize, Serialize}; 
use byte::{
    ctx::*,
    self,
    TryRead,
    // TryWrite,
    BytesExt,
};


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(from = "Dwords<N>", into = "Dwords<N>")]
pub enum BaseAddresses<const N: usize> {
    Basic([u32; N]),
    Sized([BaseAddressSized; N]),
    Resource([BaseAddressResource; N]),
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct BaseAddressSized {
    pub data: u32,
    pub size: u32,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct BaseAddressResource {
    pub start: u64,
    pub end: u64,
    pub flags: u64,
}

// Serde wrapper
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Dwords<const N: usize>(#[serde(with = "serde_arrays")] [u32; N]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseAddress {
    pub region: usize,
    pub map: BaseAddressMap,
    pub io_resource: Option<IoResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseAddressMap {
    /// 32-bit Memory Space mapping
    MemorySpace32 {
        prefetchable: bool,
        base_address: u32,
        size: Option<u32>,
    },
    /// In earlier versions it was used to support memory space below 1MB (16-bit wide base
    /// register that can be mapped anywhere in the 16-bit Memory Space). 
    MemorySpace1M {
        prefetchable: bool,
        base_address: u16,
        size: Option<u16>,
    },
    // 64-bit Memory Space mapping
    MemorySpace64 {
        prefetchable: bool,
        base_address: u64,
        size: Option<u64>,
    },
    MemorySpace64Broken,
    /// Offset for port addresses
    IoSpace {
        base_address: u32,
        size: Option<u32>,
    },
    /// bit #1 I/O space, and bits #2/1 values 01 and 11
    Reserved,
}


/// IO resources have defined flags. PCI devices expose these flags to userspace in the "resource"
/// sysfs file.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct IoResource {
    /// BAR Equivalent Indicator:  Flags && 1<<5
    pub bar_equivalent_indicator: u64,
}

impl From<u64> for IoResource {
    fn from(qword: u64) -> Self {
        Self {
            bar_equivalent_indicator: qword & (1 << 5),
        }
    }
}

#[derive(Debug, )]
pub enum Iter<'a> {
    Basic(iter::Enumerate<slice::Iter<'a, u32>>),
    Sized(iter::Enumerate<slice::Iter<'a, BaseAddressSized>>),
    Resource(iter::Enumerate<slice::Iter<'a, BaseAddressResource>>),
}


pub trait HeaderBaseAdresses {
    fn iter(&self) -> Iter;
}

impl<'a, const N: usize> TryRead<'a, Endian> for BaseAddresses<N> {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let mut array = [0u32; N];
        let iter = bytes.read_iter::<u32>(offset, endian).take(N);
        for (n, dword) in iter.enumerate() {
            array[n] = dword;
        }
        let base_addresses = BaseAddresses::Basic(array);
        Ok((base_addresses, *offset))
    }
}


impl<const N: usize> BaseAddresses<N> {
    pub fn iter(&self) -> Iter {
        match self {
            BaseAddresses::Basic(array) => Iter::Basic(array.iter().enumerate()),
            BaseAddresses::Sized(array) => Iter::Sized(array.iter().enumerate()),
            BaseAddresses::Resource(array) => Iter::Resource(array.iter().enumerate()),
        }
    }
}
impl<const N: usize> Default for BaseAddresses<N> {
    fn default() -> Self {
        BaseAddresses::Basic([Default::default(); N])
    }
}
impl<const N: usize> From<Dwords<N>> for BaseAddresses<N> {
    fn from(data: Dwords<N>) -> Self {
        BaseAddresses::Basic(data.0)
    }
}

impl<const N: usize> From<BaseAddresses<N>> for Dwords<N> {
    fn from(ba: BaseAddresses<N>) -> Self {
        let array = match ba {
            BaseAddresses::Basic(a) => a,
            BaseAddresses::Sized(a) => {
                let mut result = [0; N];
                for (i, entry) in a.iter().enumerate() {
                    result[i] = entry.data;
                }
                result
            },
            BaseAddresses::Resource(a) => {
                let mut result = [0; N];
                let mut is_addr64 = false;
                for i in 0..N {
                    if is_addr64 {
                        is_addr64 = !is_addr64;
                        continue;
                    }
                    let entry = &a[i];
                    if entry.flags & 0b111 == 0b100 {
                        result[i + 1] = (entry.start >> 32) as u32;
                        is_addr64 = true;
                    }
                    result[i] = ((entry.start & 0xFFFFFFF0) | (entry.flags & 0xF)) as u32;
                }
                result
            },
        };
        Self(array)
    }
}

impl HeaderBaseAdresses for BaseAddresses<1> {
    fn iter(&self) -> Iter { self.iter() }
}
impl HeaderBaseAdresses for BaseAddresses<2> {
    fn iter(&self) -> Iter { self.iter() }
}
impl HeaderBaseAdresses for BaseAddresses<6> {
    fn iter(&self) -> Iter { self.iter() }
}

impl<'a> Iterator for Iter<'a> {
    type Item = BaseAddress;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Basic(iter) => {
                loop {
                    let (region, &dword) = iter.next()?;
                    if dword == 0 {
                        continue;
                    }
                    let prefetchable = dword & 0b1000 != 0;
                    let base_address: u32 = dword & !0b1111;
                    let map = match dword & 0b111 {
                        0b000 => {
                            BaseAddressMap::MemorySpace32 { 
                                prefetchable,
                                base_address,
                                size: None,
                            }
                        },
                        0b010 => {
                            BaseAddressMap::MemorySpace1M { 
                                prefetchable,
                                base_address: base_address as u16,
                                size: None,
                            }
                        },
                        0b100 => {
                            if let Some((_, &dword)) = iter.next() {
                                BaseAddressMap::MemorySpace64 { 
                                    prefetchable,
                                    base_address: ((dword as u64) << 32) | (base_address as u64),
                                    size: None,
                                }
                            } else {
                                BaseAddressMap::MemorySpace64Broken
                            }
                        },
                        0b001 | 0b101 => {
                            BaseAddressMap::IoSpace {
                                base_address: dword & !0b11,
                                size: None,
                            }
                        },
                        _ => {
                            BaseAddressMap::Reserved
                        },
                    };
                    break Some(BaseAddress { region, map, io_resource: None });
                }
            },
            Self::Sized(iter) => {
                loop {
                    let (region, &BaseAddressSized { data, size }) = iter.next()?;
                    if data == 0 {
                        continue;
                    }
                    let prefetchable = data & 0b1000 != 0;
                    let base_address: u32 = data & !0b1111;
                    let map = match data & 0b111 {
                        0b000 => {
                            BaseAddressMap::MemorySpace32 { 
                                prefetchable,
                                base_address,
                                size: Some(size)
                            }
                        },
                        0b010 => {
                            BaseAddressMap::MemorySpace1M { 
                                prefetchable,
                                base_address: base_address as u16,
                                size: Some(size as u16)
                            }
                        },
                        0b100 => {
                            if let Some((_, &BaseAddressSized { data, .. })) = iter.next() {
                                BaseAddressMap::MemorySpace64 { 
                                    prefetchable,
                                    base_address: ((data as u64) << 32) | (base_address as u64),
                                    size: Some(size as u64)
                                }
                            } else {
                                BaseAddressMap::MemorySpace64Broken
                            }
                        },
                        0b001 | 0b101 => {
                            BaseAddressMap::IoSpace {
                                base_address: data & !0b11,
                                size: Some(size)
                            }
                        },
                        _ => {
                            BaseAddressMap::Reserved
                        },
                    };
                    break Some(BaseAddress { region, map, io_resource: None });
                }
            },
            Self::Resource(iter) => {
                loop {
                    let (region, &BaseAddressResource { start, end, flags }) = iter.next()?;
                    if start == 0 {
                        continue;
                    }
                    let prefetchable = flags & 0b1000 != 0;
                    let map = match flags & 0b111 {
                        0b000 => {
                            BaseAddressMap::MemorySpace32 { 
                                prefetchable,
                                base_address: start as u32,
                                size: Some((end - start + 1) as u32),
                            }
                        },
                        0b010 => {
                            BaseAddressMap::MemorySpace1M { 
                                prefetchable,
                                base_address: start as u16,
                                size: Some((end - start + 1) as u16),
                            }
                        },
                        0b100 => {
                            BaseAddressMap::MemorySpace64 { 
                                prefetchable,
                                base_address: start,
                                size: Some(end - start + 1),
                            }
                        },
                        0b001 | 0b101 => {
                            BaseAddressMap::IoSpace {
                                base_address: start as u32,
                                size: Some((end - start + 1) as u32),
                            }
                        },
                        _ => {
                            BaseAddressMap::Reserved
                        },
                    };
                    break Some(BaseAddress { region, map, io_resource: Some(flags.into()) });
                }
            },
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn dwords_wrapper() {
        let ba = BaseAddresses::Basic([0xb4418004,0xb4418004,0xb4100004]);
        assert_eq!(Dwords([0xb4418004,0xb4418004,0xb4100004]), ba.into());
        
        let ba = BaseAddresses::Sized([
            BaseAddressSized { data: 0xb4418004, size: 0 },
            BaseAddressSized { data: 0xb4418004, size: 0xff },
            BaseAddressSized { data: 0xb4100004, size: 0 },
        ]);
        assert_eq!(Dwords([0xb4418004,0xb4418004,0xb4100004]), ba.into());
        
        let ba = BaseAddresses::Resource([
            BaseAddressResource { start: 0xb3000000, end: 0xb3ffffff, flags: 0x040200 },
            BaseAddressResource { start: 0xa0000000, end: 0xafffffff, flags: 0x14220c, },
            BaseAddressResource { start: 0, end: 0, flags: 0, },
            BaseAddressResource { start: 0x3bffff1c000, end: 0x3bffff1ffff, flags: 0x140204, },
            BaseAddressResource { start: 0, end: 0, flags: 0, },
            BaseAddressResource { start: 0x00003000, end: 0x0000307f, flags: 0x040101, },
        ]);
        let sample = Dwords([
            0xb3000000,
            0xa000000c,
            0,
            0xfff1c004,
            0x000003bf,
            0x00003001,
        ]);
        assert_eq!(sample, ba.into());
        
    }

    #[test]
    fn iter_basic_once() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: None }, 
                io_resource: None
            },
        ];

        let basic = BaseAddresses::Basic([0xb3000000, 0, 0, 0, 0, 0])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, basic, "Basic");
    }

    #[test]
    fn iter_basic_full() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: None },
                io_resource: None
            },
            BaseAddress {
                region: 1,
                map: BaseAddressMap::MemorySpace64 { prefetchable: true, base_address: 0xa0000000, size: None },
                io_resource: None
            },
            BaseAddress {
                region: 3,
                map: BaseAddressMap::MemorySpace64 { prefetchable: false, base_address: 0x3bffff1c000, size: None },
                io_resource: None
            },
            BaseAddress {
                region: 5,
                map: BaseAddressMap::IoSpace { base_address: 0x3000, size: None },
                io_resource: None
            },
        ];

        let basic =
            BaseAddresses::Basic([
                0xb3000000,
                0xa000000c,
                0,
                0xfff1c004,
                0x000003bf,
                0x00003001,
            ])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, basic, "Basic");
    }

    #[test]
    fn iter_sized_once() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: Some(16 << 20) },
                io_resource: None
            },
        ];

        let sized = 
            BaseAddresses::Sized([
                BaseAddressSized { data: 0xb3000000, size: 16 << 20 },
                BaseAddressSized { data: 0, size: 0 },
                BaseAddressSized { data: 0, size: 0 },
                BaseAddressSized { data: 0, size: 0 },
                BaseAddressSized { data: 0, size: 0 },
                BaseAddressSized { data: 0, size: 0 },
            ])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, sized, "Basic");
    }

    #[test]
    fn iter_sized_full() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: Some(16 << 20) },
                io_resource: None
            },
            BaseAddress {
                region: 1,
                map: BaseAddressMap::MemorySpace64 { prefetchable: true, base_address: 0xa0000000, size: Some(256 << 20) },
                io_resource: None
            },
            BaseAddress {
                region: 3,
                map: BaseAddressMap::MemorySpace64 { prefetchable: false, base_address: 0x3bffff1c000, size: Some(16384) },
                io_resource: None
            },
            BaseAddress {
                region: 5,
                map: BaseAddressMap::IoSpace { base_address: 0x3000, size: Some(0) },
                io_resource: None
            },
        ];
        let sized = 
            BaseAddresses::Sized([
                BaseAddressSized { data: 0xb3000000, size: 16 << 20 },
                BaseAddressSized { data: 0xa000000c, size: 256 << 20 },
                BaseAddressSized { data: 0, size: 0 },
                BaseAddressSized { data: 0xfff1c004, size: 16 << 10 },
                BaseAddressSized { data: 0x000003bf, size: 0 },
                BaseAddressSized { data: 0x00003001, size: 0 },
            ])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, sized, "Sized");
    }

    #[test]
    fn iter_resource_once() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: Some(16 << 20) },
                io_resource: Some(IoResource { bar_equivalent_indicator: 0x00040200 & (1 << 5) }),
            },
        ];
        let resource = 
            BaseAddresses::Resource([
                BaseAddressResource { start: 0xb3000000, end: 0xb3ffffff, flags: 0x00040200 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
            ])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, resource, "Resource");
    }

    #[test]
    fn iter_resource_full() {
        let sample = vec![
            BaseAddress {
                region: 0,
                map: BaseAddressMap::MemorySpace32 { prefetchable: false, base_address: 0xb3000000, size: Some(16 << 20) },
                io_resource: Some(IoResource { bar_equivalent_indicator: 0x00040200 & (1 << 5) }),
            },
            BaseAddress {
                region: 1,
                map: BaseAddressMap::MemorySpace64 { prefetchable: true, base_address: 0xa0000000, size: Some(256 << 20) },
                io_resource: Some(IoResource { bar_equivalent_indicator: 0x00040200 & (1 << 5) }),
            },
            BaseAddress {
                region: 3,
                map: BaseAddressMap::MemorySpace64 { prefetchable: false, base_address: 0x3bffff1c000, size: Some(16384) },
                io_resource: Some(IoResource { bar_equivalent_indicator: 0x00040200 & (1 << 5) }),
            },
            BaseAddress {
                region: 5,
                map: BaseAddressMap::IoSpace { base_address: 0x3000, size: Some(0x7f + 1) },
                io_resource: Some(IoResource { bar_equivalent_indicator: 0x00040200 & (1 << 5) }),
            },
        ];
        let resource = 
            BaseAddresses::Resource([
                BaseAddressResource { start: 0xb3000000, end: 0xb3ffffff, flags: 0x00040200 },
                BaseAddressResource { start: 0xa0000000, end: 0xafffffff, flags: 0x0014220c },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0x3bffff1c000, end: 0x3bffff1ffff, flags: 0x140204 },
                BaseAddressResource { start: 0, end: 0, flags: 0 },
                BaseAddressResource { start: 0x00003000, end: 0x0000307f, flags: 0x00040101 },
            ])
            .iter().collect::<Vec<_>>();
        assert_eq!(sample, resource, "Resource");
    }
}
