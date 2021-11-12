use core::any::TypeId;

use super::*;
use crate::device::header::command::Command;
use crate::device::header::status::{Status, StatusType, Primary, SecondaryBridge, SecondaryCardbus};



impl<'a> fmt::Display for MultiView<&'a Command, &'a View> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
        match self.view {
            View::Basic => {
                write!(f, 
                    "I/O Space: {}, Memory Space: {}, Bus Master: {}, Special Cycles: {}, Memory Write and Invalidate Enable: {}, VGA Palette Snoop: {}, Parity Error Response: {}, Stepping: {}, SERR# Enable: {}, Fast Back-to-Back Enable: {}, Interrupt Disable: {}",
                    d.io_space.display(BoolView::PlusMinus),
                    d.memory_space.display(BoolView::PlusMinus),
                    d.bus_master.display(BoolView::PlusMinus),
                    d.special_cycles.display(BoolView::PlusMinus),
                    d.memory_write_and_invalidate_enable.display(BoolView::PlusMinus),
                    d.vga_palette_snoop.display(BoolView::PlusMinus),
                    d.parity_error_response.display(BoolView::PlusMinus),
                    d.stepping.display(BoolView::PlusMinus),
                    d.serr_enable.display(BoolView::PlusMinus),
                    d.fast_back_to_back_enable.display(BoolView::PlusMinus),
                    d.interrupt_disable.display(BoolView::PlusMinus),
                )
            },
            View::LspciBasic(_)  => {
                write!(f,
                    "I/O{} Mem{} BusMaster{} SpecCycle{} MemWINV{} VGASnoop{} ParErr{} Stepping{} SERR{} FastB2B{} DisINTx{}",
                    d.io_space.display(BoolView::PlusMinus),
                    d.memory_space.display(BoolView::PlusMinus),
                    d.bus_master.display(BoolView::PlusMinus),
                    d.special_cycles.display(BoolView::PlusMinus),
                    d.memory_write_and_invalidate_enable.display(BoolView::PlusMinus),
                    d.vga_palette_snoop.display(BoolView::PlusMinus),
                    d.parity_error_response.display(BoolView::PlusMinus),
                    d.stepping.display(BoolView::PlusMinus),
                    d.serr_enable.display(BoolView::PlusMinus),
                    d.fast_back_to_back_enable.display(BoolView::PlusMinus),
                    d.interrupt_disable.display(BoolView::PlusMinus),
                )
            },
            View::Extended => {
                write!(f,
                    "Can respond to I/O Space accesses: {}\n\
                    Can respond to Memory Space accesses: {}\n\
                     Can behave as a bus master: {}\n\
                     Can monitor Special Cycle operations: {}\n\
                     Can generate the Memory Write and Invalidate command: {}\n\
                     Does not respond to palette register writes and will snoop the data: {}\n\
                     Will take its normal action when a parity error is detected: {}\n\
                     As of revision 3.0 of the PCI local bus specification this bit is hardwired to 0. In earlier versions of the specification this bit was used by devices and may have been hardwired to 0, 1, or implemented as a read/write bit.: {}\n\
                     The SERR# driver is enabled: {}\n\
                     Indicates a device is allowed to generate fast back-to-back transactions: {}\n\
                     The assertion of the devices INTx# signal is disabled: {}\n",
                    d.io_space.display(BoolView::PlusMinus),
                    d.memory_space.display(BoolView::PlusMinus),
                    d.bus_master.display(BoolView::PlusMinus),
                    d.special_cycles.display(BoolView::PlusMinus),
                    d.memory_write_and_invalidate_enable.display(BoolView::PlusMinus),
                    d.vga_palette_snoop.display(BoolView::PlusMinus),
                    d.parity_error_response.display(BoolView::PlusMinus),
                    d.stepping.display(BoolView::PlusMinus),
                    d.serr_enable.display(BoolView::PlusMinus),
                    d.fast_back_to_back_enable.display(BoolView::PlusMinus),
                    d.interrupt_disable.display(BoolView::PlusMinus),
                )
            },
            _ => Ok(())
        }
    }
}
impl<'a> DisplayMultiView<'a> for Command {
    type Data = &'a Command;
    type View = &'a View;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}



impl<'a, T: 'static + StatusType> fmt::Display for MultiView<&'a Status<T>, &'a View> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
        match &self.view {
            View::Basic => write!(f, 
                "Interrupt Status: {}, Capabilities List: {}, 66 MHz Capable: {}, User Definable \
                Features: {}, Fast Back-to-Back Capable: {}, Master Data Parity Error: {}, DEVSEL \
                Timing: {}, Signalled Target Abort: {}, Received Target Abort: {}, Received Master \
                Abort: {}, SERR: {}, Detected Parity Error: {}",
                d.interrupt_status.display(BoolView::PlusMinus),
                d.capabilities_list.display(BoolView::PlusMinus),
                d.is_66mhz_capable.display(BoolView::PlusMinus),
                d.user_definable_features.display(BoolView::PlusMinus),
                d.fast_back_to_back_capable.display(BoolView::PlusMinus),
                d.master_data_parity_error.display(BoolView::PlusMinus),
                d.devsel_timing,
                d.signaled_target_abort.display(BoolView::PlusMinus),
                d.received_target_abort.display(BoolView::PlusMinus),
                d.received_master_abort.display(BoolView::PlusMinus),
                d.system_error.display(BoolView::PlusMinus),
                d.detected_parity_error.display(BoolView::PlusMinus),
            ),
            View::LspciBasic(_) if TypeId::of::<T>() == TypeId::of::<Primary>() => write!(f,
                "Cap{} 66MHz{} UDF{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} \
                >SERR{} <PERR{} INTx{}",
                d.capabilities_list.display(BoolView::PlusMinus),
                d.is_66mhz_capable.display(BoolView::PlusMinus),
                d.user_definable_features.display(BoolView::PlusMinus),
                d.fast_back_to_back_capable.display(BoolView::PlusMinus),
                d.master_data_parity_error.display(BoolView::PlusMinus),
                d.devsel_timing,
                d.signaled_target_abort.display(BoolView::PlusMinus),
                d.received_target_abort.display(BoolView::PlusMinus),
                d.received_master_abort.display(BoolView::PlusMinus),
                d.system_error.display(BoolView::PlusMinus),
                d.detected_parity_error.display(BoolView::PlusMinus),
                d.interrupt_status.display(BoolView::PlusMinus),
            ),
            View::LspciBasic(_) if TypeId::of::<T>() == TypeId::of::<SecondaryBridge>() => write!(f,
                "66MHz{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} <SERR{} <PERR{}",
                d.is_66mhz_capable.display(BoolView::PlusMinus),
                d.fast_back_to_back_capable.display(BoolView::PlusMinus),
                d.master_data_parity_error.display(BoolView::PlusMinus),
                d.devsel_timing,
                d.signaled_target_abort.display(BoolView::PlusMinus),
                d.received_target_abort.display(BoolView::PlusMinus),
                d.received_master_abort.display(BoolView::PlusMinus),
                d.system_error.display(BoolView::PlusMinus),
                d.detected_parity_error.display(BoolView::PlusMinus),
            ),
            View::LspciBasic(_) if TypeId::of::<T>() == TypeId::of::<SecondaryCardbus>() => {
                if d.system_error {
                    write!(f, "SERR")
                } else {
                    Ok(())
                }
            },
            View::Extended => write!(f,
                "Represents the state of the device's INTx# signal: {}\n\
                Device implements the pointer for a New Capabilities Linked list: {}\n\
                Device is capable of running at 66 MHz: {}\n\
                Support User Definable Features [obsolete]: {}\n\
                Device can accept fast back-to-back transactions that are not from the same agent: {}\n\
                The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write: {}\n\
                Represent the slow timing that a device will assert DEVSEL# for any bus command except Configuration Space read and write: {}\n\
                Target device terminates a transaction with Target-Abort: {}\n\
                Master device transaction is terminated with Target-Abort: {}\n\
                Master device transaction (except for Special Cycle transactions) is terminated with Master-Abort: {}\n\
                SERR#: {}\n\
                Device detects a parity error, even if parity error handling is disabled: {}\n",
                d.interrupt_status.display(BoolView::PlusMinus),
                d.capabilities_list.display(BoolView::PlusMinus),
                d.is_66mhz_capable.display(BoolView::PlusMinus),
                d.user_definable_features.display(BoolView::PlusMinus),
                d.fast_back_to_back_capable.display(BoolView::PlusMinus),
                d.master_data_parity_error.display(BoolView::PlusMinus),
                d.devsel_timing,
                d.signaled_target_abort.display(BoolView::PlusMinus),
                d.received_target_abort.display(BoolView::PlusMinus),
                d.received_master_abort.display(BoolView::PlusMinus),
                d.system_error.display(BoolView::PlusMinus),
                d.detected_parity_error.display(BoolView::PlusMinus),
            ),
            _ => Ok(()),
        }
    }
}
impl<'a, T: 'a + StatusType> DisplayMultiView<'a> for Status<T> {
    type Data = &'a Status<T>;
    type View = &'a View;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}

// impl BridgeControl {
//     pub const LAYOUT: [View<'static>; 16] = [
//         View { name: "Parity Error Response Enable", desc: "Controls the bridge’s response to address and data parity errors on the secondary interface", lspci: "Parity" },
//         View { name: "SERR# Enable", desc: "Controls the forwarding of secondary interface SERR# assertions to the primary interface", lspci: "SERR" },
//         View { name: "ISA Enable", desc: "Modifies the response by the bridge to ISA I/O addresses", lspci: "NoISA" },
//         View { name: "VGA Enable", desc: "Modifies the response by the bridge to VGA compatible addresses", lspci: "VGA" },
//         View { name: "VGA16", desc: "VGA16", lspci: "VGA16" },
//         View { name: "Master-Abort Mode", desc: "Controls the behavior of a bridge when a Master-Abort termination occurs on either interface while the bridge is the master of the transaction", lspci: "MAbort" },
//         View { name: "Secondary Bus Reset", desc: "Forces the assertion of RST# on the secondary interface", lspci: ">Reset" },
//         View { name: "Fast Back-toBack Enable", desc: "Controls ability of the bridge to generate fast back-to-back transactions to different devices on the secondary interface", lspci: "FastB2B" },
//         View { name: "Primary Discard Timer", desc: "Selects the number of PCI clocks that the bridge will wait for a master on the primary interface to repeat a Delayed Transaction request", lspci: "PriDiscTmr" },
//         View { name: "Secondary Discard Timer", desc: "Selects the number of PCI clocks that the bridge will wait for a master on the secondary interface to repeat a Delayed Transaction request", lspci: "SecDiscTmr" },
//         View { name: "Discard Timer Status", desc: "This bit is set when either the Primary Discard Timer or Secondary Discard Timer expires and a Delayed Completion is discarded from a queue in the bridge", lspci: "DiscTmrStat" },
//         View { name: "Discard Timer SERR# Enable", desc: "This bit enables the bridge to assert SERR# on the primary interface when either the Primary Discard Timer or Secondary Discard Timer expires and a Delayed Transaction is discarded from a queue in the bridge", lspci: "DiscTmrSERREn" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//     ];
// }



// impl CardbusBridgeControl {
//     pub const LAYOUT: [View<'static>; 16] = [
//         View { name: "Parity Error Response Enable", desc: "Controls the response to parity errors on the CardBus", lspci: "Parity" },
//         View { name: "SERR# Enable", desc: "Controls forwarding of SERR# signals indicated on the CardBus", lspci: "SERR" },
//         View { name: "ISA Enable", desc: "This applies only to addresses that are enabled by the I/O Base and Limit registers and are also in the first 64 KBytes of PCI I/O space", lspci: "ISA" },
//         View { name: "VGA Enable", desc: "Modifies the bridge's response to VGA compatible addresses", lspci: "VGA" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Master Abort Mode", desc: "Controls the behavior of the bridge when a master abort occurs on either PCI or CardBus interface when the bridge is master", lspci: "MAbort" },
//         View { name: "CardBus Reset", desc: "When set the bridge will assert and hold CRST#", lspci: ">Reset" },
//         View { name: "IREQ-INT Enable", desc: "When set this bit enables the IRQ routing register for 16-bit PC Cards", lspci: "16bInt" },
//         View { name: "Memory 0 Prefetch Enable", desc: "When set enables Read prefetching from the memory window defined to by the Memory Base 0 and Memory Limit 0 registers", lspci: "Mem0Pref" },
//         View { name: "Memory 1 Prefetch Enable", desc: "When set enables Read prefetching from the memory window defined to by the Memory Base 1 and Memory Limit 1 registers", lspci: "Mem1Pref" },
//         View { name: "Write Posting Enable", desc: "Enables posting of Write data to and from the socket", lspci: "PostWrite" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//         View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
//     ];
// }


#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn command() {
        let view = View::LspciBasic(Default::default());
        let command: Command = u16::from_le_bytes([0x06, 0x04]).into();
        assert_eq!(
            "I/O- Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx+", 
            command.display(&view).to_string()
        );
    }

    #[test]
    fn status() {
        let view = View::LspciBasic(Default::default());
        let status: Status<Primary> = u16::from_le_bytes([0x80, 0x02]).into();
        assert_eq!(
            "Cap- 66MHz- UDF- FastB2B+ ParErr- DEVSEL=medium >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-", 
            status.display(&view).to_string()
        );
    }
}

