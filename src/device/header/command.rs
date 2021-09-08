use core::array;

use serde::{Serialize, Deserialize,}; 
use bitfield_layout::{BitFieldLayout, Layout, };

use super::View;


/// Provides control over a device's ability to generate and respond to PCI cycles. Where the
/// only functionality guaranteed to be supported by all devices is, when a 0 is written to
/// this register, the device is disconnected from the PCI bus for all accesses except
/// Configuration Space access.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Command(pub u16);


impl Command {
    pub const LAYOUT: [View<'static>; 16] = [
        View { name: "I/O Space", desc: "Can respond to I/O Space accesses", lspci: "I/O" },
        View { name: "Memory Space", desc: "Can respond to Memory Space accesses", lspci: "Mem" },
        View { name: "Bus Master", desc: "Can behave as a bus master", lspci: "BusMaster" },
        View { name: "Special Cycles", desc: "Can monitor Special Cycle operations", lspci: "SpecCycle" },
        View { name: "Memory Write and Invalidate Enable", desc: "Can generate the Memory Write and Invalidate command", lspci: "MemWINV" },
        View { name: "VGA Palette Snoop", desc: "Does not respond to palette register writes and will snoop the data", lspci: "VGASnoop" },
        View { name: "Parity Error Response", desc: "Will take its normal action when a parity error is detected", lspci: "ParErr" },
        View { name: "Stepping", desc: "As of revision 3.0 of the PCI local bus specification this bit is hardwired to 0. In earlier versions of the specification this bit was used by devices and may have been hardwired to 0, 1, or implemented as a read/write bit.", lspci: "Stepping" },
        View { name: "SERR# Enable", desc: "The SERR# driver is enabled", lspci: "SERR" },
        View { name: "Fast Back-to-Back Enable", desc: "Indicates a device is allowed to generate fast back-to-back transactions", lspci: "FastB2B" },
        View { name: "Interrupt Disable", desc: "The assertion of the devices INTx# signal is disabled", lspci: "DisINTx" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
    ];
}
impl Layout for Command {
    type Layout = array::IntoIter<View<'static>, 16>;
    fn layout() -> Self::Layout { array::IntoIter::new(Self::LAYOUT) }
}
impl BitFieldLayout for Command {
    type Value = u16;
    fn get(&self) -> Self::Value { self.0 }
    fn set(&mut self, new: Self::Value) { self.0 = new; }
}
