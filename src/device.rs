//! PCI configuration space is the underlying way that the Conventional PCI, PCI-X and PCI Express
//! perform auto configuration of the cards inserted into their bus.
//!

use core::cmp::Ordering;

use displaydoc::Display;
use byte::{
    ctx::*,
    self,
    TryRead,
    // TryWrite,
    BytesExt,
};

pub mod address;
pub use address::Address;

pub mod header;
pub use header::{Header, HeaderType};

pub mod capabilities;
pub use capabilities::Capabilities;


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
        }
    }
    pub fn capabilities(&self) -> Option<Capabilities> {
        let ddr = self.device_dependent_region.as_ref();
        let ptr = self.header.capabilities_pointer;
        ddr.filter(|_| ptr > 0)
            .map(|ddr| Capabilities::new(&ddr, ptr))
    }
    pub fn irq(&self) -> u8 {
        if let Some(irq) = self.irq {
            irq
        } else {
            self.header.interrupt_line
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
        }
    }
}
impl<'a> TryRead<'a, Endian> for ConfigurationSpace {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let header = bytes.read_with::<Header>(offset, endian)?; 
        let device_dependent_region = bytes.get(64..256)
            .and_then(|bytes| bytes.try_into().ok());
        let extended_configuration_space = bytes.get(256..4096)
            .and_then(|bytes| bytes.try_into().ok());
        Ok((Self { header, device_dependent_region, extended_configuration_space }, *offset))
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


#[cfg(test)]
mod tests {
    use byte::*;
    use super::*;

    #[test]
    fn device_order() {
        let cs: ConfigurationSpace = [0; 64].read_with(&mut 0, LE).unwrap();
        let a = Device::new(Default::default(), cs.clone());
        let b = Device::new("00:00.1".parse().unwrap(), cs);
        assert!(a < b);
    }

    #[test]
    fn empty_capabilities() {
        let cs = [0; 64].read_with(&mut 0, LE).unwrap();
        let device = Device::new(Default::default(), cs);
        assert_eq!(None, device.capabilities());
    }
}
