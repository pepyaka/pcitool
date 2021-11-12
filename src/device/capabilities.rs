
use byte::{
    ctx::*,
    self,
    //TryRead,
    // TryWrite,
    BytesExt,
};


pub mod power_management_interface;
pub use power_management_interface::PowerManagementInterface;

pub mod vendor_specific;
pub use vendor_specific::VendorSpecific;

pub mod message_signaled_interrups;
pub use message_signaled_interrups::MessageSignaledInterrups;

use super::{DDR_OFFSET, DeviceDependentRegion};

#[derive(Debug, PartialEq, Eq)]
pub struct Capability<'a> {
    pub pointer: u8,
    pub kind: CapabilityKind<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityKind<'a> {
    PowerManagementInterface(PowerManagementInterface),
    // AcceleratedGraphicsPort(AcceleratedGraphicsPort),
    // VitalProductData(VitalProductData),
    // SlotIndetification(SlotIndetification),
    MessageSignaledInterrups(MessageSignaledInterrups),
    // CompactPciHotSwap(CompactPciHotSwap),
    // PciX(PciX),
    // HyperTransport(HyperTransport),
    VendorSpecific(VendorSpecific<'a>),
    // DebugPort(DebugPort),
    // CompactPciResourceControl(CompactPciResourceControl),
    // PciHotPlug(PciHotPlug),
    // PciBridgeSubsystemVendorId(PciBridgeSubsystemVendorId),
    // AcceleratedGraphicsPort8X(AcceleratedGraphicsPort8X),
    // SecureDevice(SecureDevice),
    // PciExpress(PciExpress),
    // MsiX(MsiX),
    Reserved(u8),
}

/// An iterator through *Capabilities List*
///
/// Used to point to a linked list of new capabilities implemented by this device. This
/// register is only valid if the “Capabilities List” bit in the [Status] Register is set. If
/// implemented, the bottom two bits are reserved and should be set to 00b.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities<'a> {
    ddr: &'a DeviceDependentRegion,
    pointer: u8,
}

impl<'a> Capabilities<'a> {
    pub fn new(ddr: &'a DeviceDependentRegion, pointer: u8) -> Self {
        Self { ddr, pointer }
    }
    fn offset(&self) -> Option<usize> {
        (self.pointer as usize).checked_sub(DDR_OFFSET)
    }
    fn cap_id(&self) -> Option<u8> {
        self.ddr.0.get(self.offset()?).cloned()
    }
    fn next_pointer(&self) -> Option<u8> {
        self.ddr.0.get(self.offset()? + 1).cloned()
    }
}

impl<'a> Iterator for Capabilities<'a> {
    type Item = Capability<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop iterating if next pointer is null
        if self.pointer == 0 {
            None
        } else {
            let data = &self.ddr.0;
            let start = self.offset()? + 2;
            let kind = match self.cap_id()? {
                0x01 => {
                    let pmi = data.read_with(&mut start.clone(), LE).ok()?;
                    CapabilityKind::PowerManagementInterface(pmi)
                },
                0x05 => {
                    let msi = data.read_with(&mut start.clone(), LE).ok()?;
                    CapabilityKind::MessageSignaledInterrups(msi)
                },
                0x09 => {
                    let vs = data.read_with(&mut start.clone(), LE).ok()?;
                    CapabilityKind::VendorSpecific(vs)
                },
                v => CapabilityKind::Reserved(v),
            };
            let pointer = self.pointer;
            self.pointer = self.next_pointer()?;
            Some(Capability { pointer, kind })
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::device::ECS_OFFSET;

    use super::*;
    use pretty_assertions::assert_eq;
    // use extfmt::{hexdump, AsHexdump};
    #[test]
    fn capabilities() {
        // Capabilities: [50] Power Management version 3
        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=55mA PME(D0-,D1-,D2-,D3hot+,D3cold+)
        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
        // Capabilities: [80] Vendor Specific Information: Len=14 <?>
        // Capabilities: [60] MSI: Enable+ Count=1/1 Maskable- 64bit+
        //         Address: 00000000fee00578  Data: 0000
        let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/config"));
        let ddr = data[DDR_OFFSET..ECS_OFFSET].try_into().unwrap();
        let result = Capabilities::new(&ddr, 0x50).collect::<Vec<_>>();
        use CapabilityKind::*;
        let sample = vec![
            Capability { 
                pointer: 0x50, 
                kind: PowerManagementInterface(data.read_with(&mut (0x50 + 2), LE).unwrap()) 
            },
            Capability { 
                pointer: 0x80, 
                kind: VendorSpecific(data.read_with(&mut (0x80 + 2), LE).unwrap()) 
            },
            Capability { 
                pointer: 0x60, 
                kind: MessageSignaledInterrups(data.read_with(&mut (0x60 + 2), LE).unwrap()) 
            },

        ];
        assert_eq!(sample, result);
    }

}
