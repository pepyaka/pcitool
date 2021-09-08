use core::array;

use serde::{Serialize, Deserialize,}; 
use bitfield_layout::{BitFieldLayout, Layout, };

use super::View;


/// The Status register is used to record status information for PCI bus related events. Devices
/// may not need to implement all bits, depending on device functionality. Reserved bits should be
/// read-only and return zero when read. 
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Status(pub u16);

impl Status {
    pub const LAYOUT: [View<'static>; 16] = [
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Interrupt Status", desc: "Represents the state of the device's INTx# signal", lspci: "INTx" },
        View { name: "Capabilities List", desc: "Device implements the pointer for a New Capabilities Linked list", lspci: "Cap" },
        View { name: "66 MHz Capable", desc: "Device is capable of running at 66 MHz", lspci: "66MHz" },
        View { name: "User Definable Features", desc: "Support User Definable Features [obsolete]", lspci: "UDF" },
        View { name: "Fast Back-to-Back Capable", desc: "Device can accept fast back-to-back transactions that are not from the same agent", lspci: "FastB2B" },
        View { name: "Master Data Parity Error", desc: "The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write", lspci: "ParErr" },
        View { name: "DEVSEL Medium Timing", desc: "Represent the medium timing that a device will assert DEVSEL# for any bus command except Configuration Space read and write", lspci: "DEVSEL=medium" },
        View { name: "DEVSEL Slow Timing", desc: "Represent the slow timing that a device will assert DEVSEL# for any bus command except Configuration Space read and write", lspci: "DEVSEL=slow" },
        View { name: "Signalled Target Abort", desc: "Target device terminates a transaction with Target-Abort", lspci: ">TAbort" },
        View { name: "Received Target Abort", desc: "Master device transaction is terminated with Target-Abort", lspci: "<TAbort" },
        View { name: "Received Master Abort", desc: "Master device transaction (except for Special Cycle transactions) is terminated with Master-Abort", lspci: "<MAbort" },
        View { name: "Signalled System Error", desc: "Device asserts SERR#", lspci: ">SERR" },
        View { name: "Detected Parity Error", desc: "Device detects a parity error, even if parity error handling is disabled", lspci: "<PERR" },
    ];
}
impl Layout for Status {
    type Layout = array::IntoIter<View<'static>, 16>;
    fn layout() -> Self::Layout { array::IntoIter::new(Self::LAYOUT) }
}
impl BitFieldLayout for Status {
    type Value = u16;
    fn get(&self) -> Self::Value { self.0 }
    fn set(&mut self, new: Self::Value) { self.0 = new; }
}

