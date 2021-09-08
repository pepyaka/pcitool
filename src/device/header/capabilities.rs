use core::cell::Cell;
use std::{convert::{TryFrom, TryInto}, fmt::{self, Display, Formatter}};

use crate::{DisplayView, View, device::{Header, Device}};

pub mod power_management_interface;
use power_management_interface::PowerManagementInterface;

pub mod vendor_specific;
use vendor_specific::VendorSpecific;

pub mod message_signaled_interrups;
use message_signaled_interrups::MessageSignaledInterrups;



#[derive(Debug)]
pub struct Capability<'a> {
    pub offset: usize,
    pub next: usize,
    pub kind: CapabilityKind<'a>,
    _view: Cell<View>,
}

#[derive(Debug)]
pub enum CapabilityKind<'a> {
    PowerManagementInterface(PowerManagementInterface),
    AcceleratedGraphicsPort(AcceleratedGraphicsPort),
    VitalProductData(VitalProductData),
    SlotIndetification(SlotIndetification),
    MessageSignaledInterrups(MessageSignaledInterrups),
    CompactPciHotSwap(CompactPciHotSwap),
    PciX(PciX),
    HyperTransport(HyperTransport),
    VendorSpecific(VendorSpecific<'a>),
    DebugPort(DebugPort),
    CompactPciResourceControl(CompactPciResourceControl),
    PciHotPlug(PciHotPlug),
    PciBridgeSubsystemVendorId(PciBridgeSubsystemVendorId),
    AcceleratedGraphicsPort8X(AcceleratedGraphicsPort8X),
    SecureDevice(SecureDevice),
    PciExpress(PciExpress),
    MsiX(MsiX),
    Reserved(u8),
}

#[derive(Debug)]
pub struct AcceleratedGraphicsPort;
#[derive(Debug)]
pub struct VitalProductData;
#[derive(Debug)]
pub struct SlotIndetification;
#[derive(Debug)]
pub struct CompactPciHotSwap;
#[derive(Debug)]
pub struct PciX;
#[derive(Debug)]
pub struct HyperTransport;
#[derive(Debug)]
pub struct DebugPort;
#[derive(Debug)]
pub struct CompactPciResourceControl;
#[derive(Debug)]
pub struct PciHotPlug;
#[derive(Debug)]
pub struct PciBridgeSubsystemVendorId;
#[derive(Debug)]
pub struct AcceleratedGraphicsPort8X;
#[derive(Debug)]
pub struct SecureDevice;
#[derive(Debug)]
pub struct PciExpress;
#[derive(Debug)]
pub struct MsiX;


/// An iterator through *Capabilities List*
///
/// Used to point to a linked list of new capabilities implemented by this device. This
/// register is only valid if the “Capabilities List” bit in the [Status] Register is set. If
/// implemented, the bottom two bits are reserved and should be set to 00b.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities<'a> {
    device: &'a Device,
    data: &'a [u8],
    offset: Option<usize>,
    _view: Cell<View>,
}


impl<'a> DisplayView<'a> for Capability<'a> {
    type View = &'a Capability<'a>;
    fn display(&'a self, view: View) -> Self::View {
        self._view.set(view);
        self
    }
}
impl<'a> Display for Capability<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use CapabilityKind::*;
        let view = self._view.take();
        match view {
            View::Basic => Ok(()),
            View::Lspci(_) => write!(f, "\tCapabilities: [{:02x}] ", self.offset),
            View::Extended => Ok(()),
        }?;
        match &self.kind {
            PowerManagementInterface(c) => write!(f, "{}", c.display(view)),
            VendorSpecific(c) => write!(f, "{}", c.display(view)),
            MessageSignaledInterrups(c) => write!(f, "{}", c.display(view)),
            Reserved(cid) => write!(f, "{:#02x}\n", cid),
            _ => write!(f, "\n"),
        }
    }
}

impl<'a> Capabilities<'a> {
    pub fn new(device: &'a Device, data: &'a [u8]) -> Self {
        Self {
            device,
            data,
            offset: data.get(0x34).map(|v| *v as usize),
            _view: Default::default(),
        }
    }
}

impl<'a> Iterator for Capabilities<'a> {
    type Item = Capability<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset.filter(|v| *v != 0)?;
        let capability_id = self.data.get(offset)?;
        self.offset = self.data.get(offset + 1).map(|v| *v as usize);
        let get_cap_slice = |len: usize| self.data.get(offset + 2 .. offset + 2 + len);
        let kind = match capability_id {
            0x01 => CapabilityKind::PowerManagementInterface({
                let bytes: [u8; 6] = get_cap_slice(6)?.try_into().ok()?;
                bytes.into()
            }),
            0x05 => {
                let msi = self.data[offset..].try_into().ok()?;
                CapabilityKind::MessageSignaledInterrups(msi)
            },
            0x09 => {
                let vs = self.data[offset..].try_into().ok()?;
                CapabilityKind::VendorSpecific(vs)
            },
            v => CapabilityKind::Reserved(*v),
        };
        Some(Capability { offset, next: self.offset.unwrap_or(0), kind, _view: Default::default() })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    // use extfmt::{hexdump, AsHexdump};

    #[test]
    fn capability_display_lspci() {
        // let data = include_bytes!("../../tests/data/8086:9dc8/config");
        let cap = Capability { 
            offset: 0x55,
            next: 0,
            kind: CapabilityKind::PowerManagementInterface([11,22,33,44,55,66].into()),
            _view: Cell::new(View::Lspci(0)),
        };
        assert_eq!("\tCapabilities: [55] Power Management version 3\n", format!("{}", cap))
    }


    #[test]
    fn iterator() {
        // Capabilities: [50] Power Management version 3
        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=55mA PME(D0-,D1-,D2-,D3hot+,D3cold+)
        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
        // Capabilities: [80] Vendor Specific Information: Len=14 <?>
        // Capabilities: [60] MSI: Enable+ Count=1/1 Maskable- 64bit+
        //         Address: 00000000fee00578  Data: 0000
        let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/config"));
        let header = Header::try_from(&data[..]).unwrap();
        let device = Device::new(Default::default(), header);
        let caps = Capabilities::new(&device, data);
        let s = caps.map(|c| format!("{}", c.display(View::Lspci(3))))
            .collect::<String>();
        let result = s.lines()
            .collect::<Vec<_>>();
        let sample = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/caps.lspci.vvv.txt"))
            .lines().collect::<Vec<_>>();
        assert_eq!(sample, result);
    }
}
