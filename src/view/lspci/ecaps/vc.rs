use core::fmt;

use pcics::extended_capabilities::virtual_channel::{
    VirtualChannel,
    PortVcCapability2,
    ReferenceClock,
    VcArbitrationSelect,
    PortVcStatus,
    ExtendedVirtualChannel,
    PortArbitrationSelect,
};

use crate::view::{DisplayMultiView, MultiView, BoolView};

pub struct VcView {
    pub verbose: usize,
    pub offset: u16
}

impl<'a> DisplayMultiView<VcView> for VirtualChannel<'a> {}
impl<'a> fmt::Display for MultiView<&'a VirtualChannel<'a>, VcView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let VcView { verbose, offset } = self.view;
        writeln!(f, "Virtual Channel")?;
        if verbose < 2 {
            return Ok(());
        }
        let VirtualChannel {
            port_vc_capability_1: pvcc1,
            port_vc_capability_2:
                PortVcCapability2 {
                    vc_arbitration_capability: vcap,
                    vc_arbitration_table_offset: arb_table_pos,
                },
            port_vc_control: ctrl,
            port_vc_status:
                PortVcStatus {
                    vc_arbitration_table_status: status,
                },
            ..
        } = &self.data;
        writeln!(
            f,
            "\t\tCaps:\tLPEVC={} RefClk={} PATEntryBits={}",
            pvcc1.low_priority_extended_vc_count,
            pvcc1.reference_clock.display(()),
            pvcc1.port_arbitration_table_entry_size.bits(),
        )?;
            
        let arb_4 = if vcap.reserved & 0b0001 != 0 { " ??4+" } else { "" }; 
        let arb_5 = if vcap.reserved & 0b0010 != 0 { " ??5+" } else { "" }; 
        let arb_6 = if vcap.reserved & 0b0100 != 0 { " ??6+" } else { "" }; 
        let arb_7 = if vcap.reserved & 0b1000 != 0 { " ??7+" } else { "" }; 
        write!(
            f,
            "\t\tArb:\tFixed{} WRR32{} WRR64{} WRR128{}{}{}{}{}",
            vcap.hardware_fixed_arbitration.display(BoolView::PlusMinus),
            vcap.wrr_32_phases.display(BoolView::PlusMinus),
            vcap.wrr_64_phases.display(BoolView::PlusMinus),
            vcap.wrr_128_phases.display(BoolView::PlusMinus),
            arb_4, arb_5, arb_6, arb_7,
        )?;
        write!(
            f,
            "\n\t\tCtrl:\tArbSelect={}\n",
            ctrl.vc_arbitration_select.display(())
        )?;
        writeln!(
            f,
            "\t\tStatus:\tInProgress{}",
            status.display(BoolView::PlusMinus)
        )?;
        if *arb_table_pos != 0 {
            // Should be "VC Arbitration Table" ?
            writeln!(
                f,
                "\t\tPort Arbitration Table [{:x}] <?>",
                offset + 16 * (*arb_table_pos as u16)
            )?;
        }
        for (n, evc) in self.data.extended_virtual_channels().enumerate() {
            write!(f, "\t\tVC{}:\t", n)?;
            if let Ok(ExtendedVirtualChannel {
                vc_resource_capability: caps,
                vc_resource_control: ctrl,
                vc_resource_status: sta,
            }) = evc
            {
                writeln!(
                    f,
                    "Caps:\tPATOffset={:02x} MaxTimeSlots={} RejSnoopTrans{}",
                    caps.port_arbitration_table_offset,
                    // caps.maximum_time_slots + 1,
                    // There is a bug in lspci: this value should be 7 bits 
                    // ls-ecaps.c: BITS(rcap, 16, 6) + 1
                    (caps.maximum_time_slots & 0b11_1111) + 1,
                    caps.reject_snoop_transactions.display(BoolView::PlusMinus),
                )?;
                let pac = caps.port_arbitration_capability;
                let pac_6 = if pac.reserved & 0b01 != 0 { " ??6+" } else { "" }; 
                let pac_7 = if pac.reserved & 0b10 != 0 { " ??7+" } else { "" }; 
                write!(
                    f,
                    "\t\t\tArb:\tFixed{} WRR32{} WRR64{} WRR128{} TWRR128{} WRR256{}{}{}",
                    pac.hardware_fixed_arbitration.display(BoolView::PlusMinus),
                    pac.wrr_32_phases.display(BoolView::PlusMinus),
                    pac.wrr_64_phases.display(BoolView::PlusMinus),
                    pac.wrr_128_phases.display(BoolView::PlusMinus),
                    pac.time_based_wrr_128_phases.display(BoolView::PlusMinus),
                    pac.wrr_256_phases.display(BoolView::PlusMinus),
                    pac_6, pac_7,
                )?;
                write!(
                    f,
                    "\n\t\t\tCtrl:\tEnable{} ID={} ArbSelect={} TC/VC={:02x}\n",
                    ctrl.vc_enable.display(BoolView::PlusMinus),
                    ctrl.vc_id,
                    ctrl.port_arbitration_select.display(()),
                    ctrl.tc_or_vc_map,
                )?;
                writeln!(
                    f,
                    "\t\t\tStatus:\tNegoPending{} InProgress{}",
                    sta.vc_negotiation_pending.display(BoolView::PlusMinus),
                    sta.port_arbitration_table_status
                        .display(BoolView::PlusMinus),
                )?;
                if caps.port_arbitration_table_offset != 0 {
                    writeln!(f, "\t\t\tPort Arbitration Table <?>")?;
                }
            } else {
                writeln!(f, "<unreadable>")?;
            }
        }
        Ok(())
    }
}



impl<'a> DisplayMultiView<()> for ReferenceClock {}
impl<'a> fmt::Display for MultiView<&'a ReferenceClock, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ReferenceClock::Rc100ns => write!(f, "100ns"),
            ReferenceClock::Reserved(v) => write!(f, "??{}", v),
        }
    }
}

impl<'a> DisplayMultiView<()> for VcArbitrationSelect {}
impl<'a> fmt::Display for MultiView<&'a VcArbitrationSelect, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            VcArbitrationSelect::HardwareFixedArbitration => write!(f, "Fixed"),
            VcArbitrationSelect::Wrr32phases => write!(f, "WRR32"),
            VcArbitrationSelect::Wrr64phases => write!(f, "WRR64"),
            VcArbitrationSelect::Wrr128phases => write!(f, "WRR128"),
            VcArbitrationSelect::Reserved(n) => write!(f, "??{}", n),
        }
    }
}

impl<'a> DisplayMultiView<()> for PortArbitrationSelect {}
impl<'a> fmt::Display for MultiView<&'a PortArbitrationSelect, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            PortArbitrationSelect::HardwareFixedArbitration => write!(f, "Fixed"),
            PortArbitrationSelect::Wrr32phases => write!(f, "WRR32"),
            PortArbitrationSelect::Wrr64phases => write!(f, "WRR64"),
            PortArbitrationSelect::Wrr128phases => write!(f, "WRR128"),
            PortArbitrationSelect::TimeBasedWrr128phases => write!(f, "TWRR128"),
            PortArbitrationSelect::Wrr256phases => write!(f, "WRR256"),
            PortArbitrationSelect::Reserved(n) => write!(f, "??{}", n),
        }
    }
}
