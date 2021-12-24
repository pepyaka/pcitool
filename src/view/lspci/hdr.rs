use core::fmt;

use pcics::header::{Command, Primary, SecondaryCardbus, SecondaryBridge, Status};

use crate::view::{DisplayMultiViewBasic, MultiView, BoolView};

impl<'a> DisplayMultiViewBasic<()> for Command {}
impl<'a> fmt::Display for MultiView<&'a Command, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
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
    }
}

impl<'a> DisplayMultiViewBasic<()> for Status<Primary> {}
impl<'a> fmt::Display for MultiView<&'a Status<Primary>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
        write!(f,
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
        )
    }
}

impl<'a> DisplayMultiViewBasic<()> for Status<SecondaryBridge> {}
impl<'a> fmt::Display for MultiView<&'a Status<SecondaryBridge>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
        write!(f,
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
        )
    }
}

impl<'a> DisplayMultiViewBasic<()> for Status<SecondaryCardbus> {}
impl<'a> fmt::Display for MultiView<&'a Status<SecondaryCardbus>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.data;
        if d.system_error {
            write!(f, "SERR")
        } else {
            Ok(())
        }
    }
}
