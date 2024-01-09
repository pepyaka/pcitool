/*!
# PCI Device

*/

use std::{
    array::TryFromSliceError, cmp::Ordering, num::ParseIntError, slice::SliceIndex, str::FromStr,
};

use heterob::Seq;

pub mod address;
pub use address::Address;

use pcics::{
    capabilities::Capabilities,
    extended_capabilities::ExtendedCapabilities,
    header::{BaseAddress, BaseAddressType, Bridge, Cardbus, Header, HeaderType, Normal},
};

/// Device dependent region starts at 0x40 offset
pub const DDR_OFFSET: usize = 0x40;
/// Extended configuration space starts at 0x100 offset
pub const ECS_OFFSET: usize = 0x100;

const DDR_LENGTH: usize = ECS_OFFSET - DDR_OFFSET;
const ECS_LENGTH: usize = 4096 - ECS_OFFSET;

/// Device
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub irq: Option<usize>,
    pub resource: Option<Resource>,
    /// Device handling kernel driver
    pub driver_in_use: Option<String>,
    /// Device handling capable kernel modules
    pub kernel_modules: Option<Vec<String>>,
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
            driver_in_use: None,
            kernel_modules: None,
        }
    }
    pub fn capabilities(&self) -> Option<Capabilities> {
        let Device {
            device_dependent_region,
            header,
            ..
        } = self;
        device_dependent_region
            .as_ref()
            .map(|DeviceDependentRegion(ddr)| Capabilities::new(ddr, header))
    }
    pub fn extended_capabilities(&self) -> Option<ExtendedCapabilities> {
        self.extended_configuration_space
            .as_ref()
            .map(|ecs| ExtendedCapabilities::new(&ecs.0))
    }
    pub fn irq(&self) -> usize {
        self.irq.unwrap_or(self.header.interrupt_line as usize)
    }
    pub fn has_mem_bar(&self) -> bool {
        let is_mem_bar = |ba: BaseAddress| {
            let is_non_zero_size = self
                .resource
                .as_ref()
                .and_then(|r| r.entries.get(ba.region))
                .filter(|e| e.size() > 0)
                .is_some();
            let is_non_io_space = !matches!(ba.base_address_type, BaseAddressType::IoSpace { .. });
            is_non_zero_size && is_non_io_space
        };
        match &self.header.header_type {
            HeaderType::Normal(Normal { base_addresses, .. }) => {
                base_addresses.clone().any(is_mem_bar)
            }
            HeaderType::Bridge(Bridge { base_addresses, .. }) => {
                base_addresses.clone().any(is_mem_bar)
            }
            HeaderType::Cardbus(Cardbus { base_addresses, .. }) => {
                base_addresses.clone().any(is_mem_bar)
            }
            _ => false,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDependentRegion(pub [u8; DDR_LENGTH]);

impl DeviceDependentRegion {
    pub const OFFSET: usize = 0x40;
    pub const SIZE: usize =
        ConfigurationSpace::SIZE - ExtendedConfigurationSpace::SIZE - Self::OFFSET;
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>,
    {
        self.0.get(index)
    }
}

impl<'a> TryFrom<&'a [u8]> for DeviceDependentRegion {
    type Error = core::array::TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes.try_into().map(Self)
    }
}

/// PCI Express extends the Configuration Space to 4096 bytes per Function as compared to 256 bytes
/// allowed by PCI Local Bus Specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedConfigurationSpace(pub [u8; ECS_LENGTH]);

impl ExtendedConfigurationSpace {
    pub const OFFSET: usize = 0x100;
    pub const SIZE: usize = ConfigurationSpace::SIZE - Self::OFFSET;
}

impl<'a> TryFrom<&'a [u8]> for ExtendedConfigurationSpace {
    type Error = core::array::TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes.try_into().map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationSpace {
    pub header: Header,
    pub device_dependent_region: Option<DeviceDependentRegion>,
    pub extended_configuration_space: Option<ExtendedConfigurationSpace>,
}

impl ConfigurationSpace {
    pub const SIZE: usize = 4096;
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
            driver_in_use: None,
            kernel_modules: None,
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

/// Sysfs `/sys/bus/pci/devices/*/resource` files support
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resource {
    pub entries: [ResourceEntry; 6],
    pub rom_entry: ResourceEntry,
}

impl FromStr for Resource {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut entries = [ResourceEntry::default(); 6];
        let mut lines = s.lines();
        for (re, line) in &mut entries.iter_mut().zip(&mut lines) {
            *re = line.parse()?;
        }
        let rom_entry = lines.next().unwrap_or("0x0").parse()?;
        Ok(Self { entries, rom_entry })
    }
}

/// Entry (line) of `/sys/bus/pci/devices/*/resource` files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    pub fn flags(&self) -> u64 {
        self.flags & 0xf
    }
    pub fn base_addr(&self) -> u64 {
        self.start | self.flags()
    }
}

impl FromStr for ResourceEntry {
    type Err = ParseIntError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut fields = line.split_ascii_whitespace();
        let start = fields.next().unwrap_or_default().trim_start_matches("0x");
        let end = fields.next().unwrap_or_default().trim_start_matches("0x");
        let flags = fields.next().unwrap_or_default().trim_start_matches("0x");
        Ok(Self {
            start: u64::from_str_radix(start, 16)?,
            end: u64::from_str_radix(end, 16)?,
            flags: u64::from_str_radix(flags, 16)?,
        })
    }
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
