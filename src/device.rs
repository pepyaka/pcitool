//! PCI configuration space is the underlying way that the Conventional PCI, PCI-X and PCI Express
//! perform auto configuration of the cards inserted into their bus.
//!

use core::cmp::Ordering;
use std::array::TryFromSliceError;

use displaydoc::Display;
use heterob::Seq;

pub mod address;
pub use address::Address;

use pcics::header::BaseAddressType;
pub use pcics::header::{self, Header, HeaderType};
// pub mod header;
// pub use header::{Header, HeaderType};

pub use pcics::capabilities::{self, Capabilities};
// pub mod capabilities;
// pub use capabilities::Capabilities;

pub use pcics::extended_capabilities::{self, ExtendedCapabilities};
// pub mod extended_capabilities;
// pub use extended_capabilities::ExtendedCapabilities;



/// Device dependent region starts at 0x40 offset
pub const DDR_OFFSET: usize = 0x40;
/// Extended configuration space starts at 0x100 offset
pub const ECS_OFFSET: usize = 0x100;

const DDR_LENGTH: usize = ECS_OFFSET - DDR_OFFSET;
const ECS_LENGTH: usize = 4096 - ECS_OFFSET;


#[derive(Debug, Clone, PartialEq, Eq,)] 
pub struct Device {
    pub address: Address,
    pub header: Header,
    pub device_dependent_region: Option<DeviceDependentRegion>,
    pub extended_configuration_space: Option<ExtendedConfigurationSpace>,
    /// Device name as exported by BIOS
    pub label: Option<String>,
    /// Physical slot
    pub phy_slot: Option<String>,
    /// NUMA node
    pub numa_node: Option<u16>,
    /// IOMMU group
    pub iommu_group: Option<String>,
    /// Real IRQ
    pub irq: Option<u8>,
    pub resource: Option<Resource>,
}
impl Device {
    pub fn new(address: Address, cs: ConfigurationSpace) -> Self {
        Self { 
            address, 
            header: cs.header, 
            device_dependent_region: cs.device_dependent_region,
            extended_configuration_space: cs.extended_configuration_space,
            label: None, 
            phy_slot: None, 
            numa_node: None, 
            iommu_group: None,
            irq: None,
            resource: None,
        }
    }
    pub fn capabilities(&self) -> Option<Capabilities> {
        let ddr = self.device_dependent_region.as_ref();
        ddr.filter(|_| self.header.capabilities_pointer > 0)
            .map(|ddr| Capabilities::new(&ddr.0, &self.header))
    }
    pub fn extended_capabilities(&self) -> Option<ExtendedCapabilities> {
        self.extended_configuration_space.as_ref()
            .map(|ecs| ExtendedCapabilities::new(&ecs.0))
    }
    pub fn irq(&self) -> u8 {
        if let Some(irq) = self.irq {
            irq
        } else {
            self.header.interrupt_line
        }
    }
    pub fn has_mem_bar(&self) -> bool {
        if let Some(mut base_addresses) = self.header.header_type.base_addresses() {
            base_addresses.any(|ba| {
                let is_non_zero_size = self.resource.as_ref()
                    .and_then(|r| r.entries.get(ba.region))
                    .filter(|e| e.size() > 0).is_some();
                let is_non_io_space = !matches!(ba.base_address_type, BaseAddressType::IoSpace { .. });
                is_non_zero_size && is_non_io_space
            })
        } else {
            false
        }
    }
}
impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.address.partial_cmp(&other.address)
    }
}
impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}


/// The device dependent region contains device specific information.
/// The last 48 DWORDs of the PCI configuration space.
#[derive(Debug, Clone, PartialEq, Eq,)] 
pub struct DeviceDependentRegion(pub [u8; DDR_LENGTH]);
impl<'a> TryFrom<&'a [u8]> for DeviceDependentRegion {
    type Error = core::array::TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes.try_into().map(Self)
    }
}

/// PCI Express extends the Configuration Space to 4096 bytes per Function as compared to 256 bytes
/// allowed by PCI Local Bus Specification.
#[derive(Debug, Clone, PartialEq, Eq,)] 
pub struct ExtendedConfigurationSpace(pub [u8; ECS_LENGTH]);
impl<'a> TryFrom<&'a [u8]> for ExtendedConfigurationSpace {
    type Error = core::array::TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes.try_into().map(Self)
    }
}


#[derive(Debug, Clone, PartialEq, Eq,)] 
pub struct ConfigurationSpace {
    pub header: Header,
    pub device_dependent_region: Option<DeviceDependentRegion>,
    pub extended_configuration_space: Option<ExtendedConfigurationSpace>,
}

impl ConfigurationSpace {
    pub fn device(self, address: Address) -> Device {
        Device { 
            address, 
            header: self.header, 
            device_dependent_region: self.device_dependent_region,
            extended_configuration_space: self.extended_configuration_space,
            label: None, 
            phy_slot: None, 
            numa_node: None, 
            iommu_group: None,
            irq: None,
            resource: None,
        }
    }
}
impl TryFrom<&[u8]> for ConfigurationSpace {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let Seq { head, tail } = slice.try_into()?;
        let (ddr, ecs) = if let Ok(Seq { head: ddr, tail }) = TryFrom::<&[u8]>::try_from(tail) {
            if let Ok(Seq { head: ecs, .. }) = TryFrom::<&[u8]>::try_from(tail) {
                (Some(ddr), Some(ecs))
            } else {
                (Some(ddr), None)
            }
        } else {
            (None, None)
        };
        Ok(Self {
            header: From::<[u8; Header::TOTAL_SIZE]>::from(head),
            device_dependent_region: ddr.map(DeviceDependentRegion),
            extended_configuration_space: ecs.map(ExtendedConfigurationSpace),
        })
    }
}

#[derive(Display, Debug, Clone, PartialEq, Eq,)] 
pub enum ConfigurationSpaceSize {
    /// 64-bytes predefined header region
    Header = 64,
    /// 256-bytes device dependent region
    DeviceDependentRegion = 256,
    /// 4096-bytes extended configuration space
    ExtendedConfigurationSpace = 4096,
}

impl From<usize> for ConfigurationSpaceSize {
    fn from(size: usize) -> Self {
        match size {
            0..=63   => Self::Header,
            64..=255 => Self::DeviceDependentRegion,
            _        => Self::ExtendedConfigurationSpace,
        }
    }
}

/// Sysfs `/sys/bus/pci/devices/*/resource` file support
#[derive(Display, Debug, Clone, PartialEq, Eq,)] 
pub struct Resource {
    pub entries: [ResourceEntry; 6],
    pub rom_entry: ResourceEntry,
}

#[derive(Display, Debug, Clone, PartialEq, Eq,)] 
pub struct ResourceEntry {
    pub start: u64,
    pub end: u64,
    /// PCI_IORESOURCE_* flags for regions or for expansion ROM
    pub flags: u64,
}
impl ResourceEntry {
    pub fn size(&self) -> u64 { 
        if self.end > self.start {
            self.end - self.start + 1
        } else {
            0
        }
    }
    pub fn flags(&self) -> u64 { self.flags & 0xf }
    pub fn base_addr(&self) -> u64 { self.start | self.flags() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_order() {
        let cs: ConfigurationSpace = [0; 64].as_slice().try_into().unwrap();
        let a = Device::new(Default::default(), cs.clone());
        let b = Device::new("00:00.1".parse().unwrap(), cs);
        assert!(a < b);
    }

    #[test]
    fn empty_capabilities() {
        let cs: ConfigurationSpace = [0; 64].as_slice().try_into().unwrap();
        let device = Device::new(Default::default(), cs);
        assert_eq!(None, device.capabilities());
    }
}
