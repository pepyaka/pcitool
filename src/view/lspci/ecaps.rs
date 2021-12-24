use core::fmt;

use pcics::{
    capabilities::pci_express,
    extended_capabilities::{
        ExtendedCapability,
        ExtendedCapabilityKind,
        DeviceSerialNumber,
        LatencyToleranceReporting,
        SecondaryPciExpress,
        L1PmSubstates,
        ProcessAddressSpaceId,
        VendorSpecificExtendedCapability,
        AddressTranslationServices, PageRequestInterface, AccessControlServices,
        PrecisionTimeMeasurement, DownstreamPortContainment, PowerBudgeting,
    }
};

use crate::{
    device::Device,
    view::{
        MultiView,
        DisplayMultiViewBasic, Verbose, BoolView,
    }
};

use self::aer::AerView;

use super::BasicView;


pub struct EcapsView<'a> {
    pub view: &'a BasicView,
    pub device: &'a Device,
}


impl<'a>DisplayMultiViewBasic<EcapsView<'a>> for ExtendedCapability<'a> {}
impl<'a> fmt::Display for MultiView<&'a ExtendedCapability<'a>, EcapsView<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let EcapsView { view, device } = self.view;
        let verbose = view.verbose;
        write!(f, "\tCapabilities: [{:03x}", self.data.offset)?;
        if view.verbose > 1 {
            write!(f, " v{}", self.data.version)?;
        }
        write!(f, "] ")?;

        match &self.data.kind {
            ExtendedCapabilityKind::Null =>
                writeln!(f, "Null"),
            ExtendedCapabilityKind::AdvancedErrorReporting(c) => {
                use pci_express::DeviceType::{RootPort, RootComplexEventCollector};
                let is_type_root = matches!(
                    device.pci_express_device_type(),
                    Some(RootPort | RootComplexEventCollector)
                );
                write!(f, "{}", c.display(AerView { verbose, is_type_root }))
            },
            ExtendedCapabilityKind::VirtualChannel(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::PrecisionTimeMeasurement(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::DeviceSerialNumber(c) =>
                write!(f, "{}", c.display(())),
            ExtendedCapabilityKind::PowerBudgeting(c) =>
                write!(f, "{}", c.display(())),
            ExtendedCapabilityKind::VendorSpecificExtendedCapability(c) =>
                write!(f, "{}", c.display(())),
            ExtendedCapabilityKind::AccessControlServices(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::AddressTranslationServices(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::PageRequestInterface(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::LatencyToleranceReporting(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::SecondaryPciExpress(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::ProcessAddressSpaceId(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::DownstreamPortContainment(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            ExtendedCapabilityKind::L1PmSubstates(c) =>
                write!(f, "{}", c.display(Verbose(verbose))),
            _ => writeln!(f, "TODO {:?}", &self.data.kind),
        }
    }
}

// 0001h Advanced Error Reporting (AER)
mod aer;

// 0002h Virtual Channel (VC)
mod vc;

// 0003h Device Serial Number
impl<'a> DisplayMultiViewBasic<()> for DeviceSerialNumber {}
impl<'a> fmt::Display for MultiView<&'a DeviceSerialNumber, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DeviceSerialNumber { lower_dword, upper_dword } = self.data;
        if lower_dword == &0 && upper_dword == &0 {
            Ok(())
        } else {
            let [b0, b1, b2, b3] = lower_dword.to_le_bytes();
            let [b4, b5, b6, b7] = upper_dword.to_le_bytes();
            writeln!(f,
                "Device Serial Number {:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
                b7, b6, b5, b4, b3, b2, b1, b0
            )
        }
    }
}

// 0004h Power Budgeting
impl<'a> DisplayMultiViewBasic<()> for PowerBudgeting {}
impl<'a> fmt::Display for MultiView<&'a PowerBudgeting, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "Power Budgeting <?>")
    }
}

/// 000Bh Vendor-Specific Extended Capability (VSEC)
impl<'a> DisplayMultiViewBasic<()> for VendorSpecificExtendedCapability<'a> {}
impl<'a> fmt::Display for MultiView<&'a VendorSpecificExtendedCapability<'a>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = &self.data.header;
        write!(f, "Vendor Specific Information: ")?;
        if h.vsec_length <= 8 {
            writeln!(f, "<unreadable>")
        } else {
            writeln!(f, "ID={:04x} Rev={} Len={:03x} <?>", h.vsec_id, h.vsec_rev, h.vsec_length)
        }
    }
}

/// 000Dh Access Control Services (ACS) 
impl<'a> DisplayMultiViewBasic<Verbose> for AccessControlServices<'a> {}
impl<'a> fmt::Display for MultiView<&'a AccessControlServices<'a,>, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Access Control Services")?;
        if verbose < 2 {
            return Ok(());
        }
        let AccessControlServices {
            acs_capability,
            acs_control,
            ..
        } = self.data;
        writeln!(f,
            "\t\tACSCap:\tSrcValid{} TransBlk{} ReqRedir{} CmpltRedir{} UpstreamFwd{} EgressCtrl{} DirectTrans{}", 
            acs_capability.acs_source_validation.display(BoolView::PlusMinus),
            acs_capability.acs_translation_blocking.display(BoolView::PlusMinus),
            acs_capability.acs_p2p_request_redirect.display(BoolView::PlusMinus),
            acs_capability.acs_p2p_completion_redirect.display(BoolView::PlusMinus),
            acs_capability.acs_upstream_forwarding.display(BoolView::PlusMinus),
            acs_capability.acs_p2p_egress_control.display(BoolView::PlusMinus),
            acs_capability.acs_direct_translated_p2p.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tACSCtl:\tSrcValid{} TransBlk{} ReqRedir{} CmpltRedir{} UpstreamFwd{} EgressCtrl{} DirectTrans{}", 
            acs_control.acs_source_validation_enable.display(BoolView::PlusMinus),
            acs_control.acs_translation_blocking_enable.display(BoolView::PlusMinus),
            acs_control.acs_p2p_request_redirect_enable.display(BoolView::PlusMinus),
            acs_control.acs_p2p_completion_redirect_enable.display(BoolView::PlusMinus),
            acs_control.acs_upstream_forwarding_enable.display(BoolView::PlusMinus),
            acs_control.acs_p2p_egress_control_enable.display(BoolView::PlusMinus),
            acs_control.acs_direct_translated_p2p_enable.display(BoolView::PlusMinus),
        )
    }
}

/// 000Fh Address Translation Services (ATS) 
impl<'a> DisplayMultiViewBasic<Verbose> for AddressTranslationServices {}
impl<'a> fmt::Display for MultiView<&'a AddressTranslationServices, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Address Translation Service (ATS)")?;
        if verbose < 2 {
            return Ok(());
        }
        let AddressTranslationServices {
            ats_capability: caps,
            ats_control: ctrl,
        } = self.data;
        writeln!(f,
            "\t\tATSCap:\tInvalidate Queue Depth: {:02x}", 
            caps.invalidate_queue_depth,
        )?;
        writeln!(f,
            "\t\tATSCtl:\tEnable{}, Smallest Translation Unit: {:02x}",
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.smallest_translation_unit,
        )
    }
}

/// 0013h Page Request Interface (PRI)
impl<'a> DisplayMultiViewBasic<Verbose> for PageRequestInterface {}
impl<'a> fmt::Display for MultiView<&'a PageRequestInterface, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Page Request Interface (PRI)")?;
        if verbose < 2 {
            return Ok(());
        }
        let PageRequestInterface {
            page_request_control: ctrl,
            page_request_status: sta,
            outstanding_page_request_capacity,
            outstanding_page_request_allocation,
        } = self.data;
        writeln!(f,
            "\t\tPRICtl: Enable{} Reset{}",
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.reset.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tPRISta: RF{} UPRGI{} Stopped{}",
            sta.response_failure.display(BoolView::PlusMinus),
            sta.unexpected_page_request_group_index.display(BoolView::PlusMinus),
            sta.stopped.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tPage Request Capacity: {:08x}, Page Request Allocation: {:08x}",
            outstanding_page_request_capacity,
            outstanding_page_request_allocation,
        )
    }
}

// 0018h Latency Tolerance Reporting (LTR)
impl<'a> DisplayMultiViewBasic<Verbose> for LatencyToleranceReporting {}
impl<'a> fmt::Display for MultiView<&'a LatencyToleranceReporting, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Latency Tolerance Reporting")?;
        if verbose < 2 {
            return Ok(());
        }
        let LatencyToleranceReporting {
            max_snoop_latency: snoop,
            max_no_snoop_latency: nosnoop
        } = self.data;

        writeln!(f, "\t\tMax snoop latency: {}ns", snoop.value())?;
        writeln!(f, "\t\tMax no snoop latency: {}ns", nosnoop.value())
    }
}

// 0019h Secondary PCI Express
impl<'a> DisplayMultiViewBasic<Verbose> for SecondaryPciExpress<'a> {}
impl<'a> fmt::Display for MultiView<&'a SecondaryPciExpress<'a>, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Secondary PCI Express")?;
        if verbose < 2 {
            return Ok(());
        }
        let SecondaryPciExpress {
            link_control_3: ctrl,
            lane_error_status,
            ..
        } = self.data;
        writeln!(f,
            "\t\tLnkCtl3: LnkEquIntrruptEn{} PerformEqu{}",
            ctrl.link_equalization_request_interrupt_enable.display(BoolView::PlusMinus),
            ctrl.perform_equalization.display(BoolView::PlusMinus),
        )?;
        let mut lane_err_sta = lane_error_status.0;
        write!(f, "\t\tLaneErrStat: ")?;
        if lane_err_sta > 0 {
            write!(f, "LaneErr at lane:")?;
            for n in 0.. {
                if lane_err_sta == 0 {
                    break;
                }
                if lane_err_sta & 1 != 0 {
                    write!(f, " {}", n)?;
                }
                lane_err_sta >>= 1;
            }
        } else {
            write!(f, "0")?;
        }
        writeln!(f)
    }
}

// 001Bh Process Address Space ID (PASID) 
impl<'a> DisplayMultiViewBasic<Verbose> for ProcessAddressSpaceId {}
impl<'a> fmt::Display for MultiView<&'a ProcessAddressSpaceId, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Process Address Space ID (PASID)")?;
        if verbose < 2 {
            return Ok(());
        }
        let ProcessAddressSpaceId {
            pacid_capability: caps,
            pacid_control: ctrl,
        } = self.data;
        writeln!(f,
            "\t\tPASIDCap: Exec{} Priv{}, Max PASID Width: {:02x}",
            caps.execute_permission_supported.display(BoolView::PlusMinus),
            caps.privileged_mode_supported.display(BoolView::PlusMinus),
            caps.max_pasid_width,
        )?;
        writeln!(f,
            "\t\tPASIDCtl: Enable{} Exec{} Priv{}",
            ctrl.pasid_enable.display(BoolView::PlusMinus),
            ctrl.execute_permission_enable.display(BoolView::PlusMinus),
            ctrl.privileged_mode_enable.display(BoolView::PlusMinus),
        )?;
        Ok(())
    }
}

// 001Dh Downstream Port Containment (DPC)
impl<'a> DisplayMultiViewBasic<Verbose> for DownstreamPortContainment {}
impl<'a> fmt::Display for MultiView<&'a DownstreamPortContainment, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Downstream Port Containment")?;
        if verbose < 2 {
            return Ok(());
        }
        let DownstreamPortContainment {
            dpc_capability,
            dpc_control,
            dpc_status,
            dpc_error_source_id,
            ..
        } = self.data;
        if (0, 0, 0, &0) ==
            (dpc_capability.into(), dpc_control.into(), dpc_status.into(), dpc_error_source_id)
        {
            return Ok(());
        }
        writeln!(f,
            "\t\tDpcCap:\tINT Msg #{}, RPExt{} PoisonedTLP{} SwTrigger{} RP PIO Log {}, DL_ActiveErr{}",
            dpc_capability.dpc_interrupt_message_number,
            dpc_capability.rp_extensions_for_dpc.display(BoolView::PlusMinus),
            dpc_capability.poisoned_tlp_egress_blocking_supported.display(BoolView::PlusMinus),
            dpc_capability.dpc_software_triggering_supported.display(BoolView::PlusMinus),
            dpc_capability.rp_pio_log_size,
            dpc_capability.dl_active_err_cor_signaling_supported.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tDpcCtl:\tTrigger:{:x} Cmpl{} INT{} ErrCor{} PoisonedTLP{} SwTrigger{} DL_ActiveErr{}",
            dpc_control.dpc_trigger_enable as usize,
            dpc_control.dpc_completion_control.display(BoolView::PlusMinus),
            dpc_control.dpc_interrupt_enable.display(BoolView::PlusMinus),
            dpc_control.dpc_err_cor_enable.display(BoolView::PlusMinus),
            dpc_control.poisoned_tlp_egress_blocking_enable.display(BoolView::PlusMinus),
            dpc_control.dpc_software_trigger.display(BoolView::PlusMinus),
            dpc_control.dl_active_err_cor_enable.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tDpcSta:\tTrigger{} Reason:{:02x} INT{} RPBusy{} TriggerExt:{:02x} RP PIO ErrPtr:{:02x}",
            dpc_status.dpc_trigger_status.display(BoolView::PlusMinus),
            dpc_status.dpc_trigger_reason.value(),
            dpc_status.dpc_interrupt_status.display(BoolView::PlusMinus),
            dpc_status.dpc_rp_busy.display(BoolView::PlusMinus),
            dpc_status.dpc_trigger_reason.extension_value(),
            dpc_status.rp_pio_first_error_pointer,
        )?;
        writeln!(f, "\t\tSource:\t{:04x}", dpc_error_source_id)
    }
}

// 001Eh L1 PM Substates
impl<'a> DisplayMultiViewBasic<Verbose> for L1PmSubstates {}
impl<'a> fmt::Display for MultiView<&'a L1PmSubstates, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "L1 PM Substates")?;
        if verbose < 2 {
            return Ok(());
        }
        let L1PmSubstates {
            l1_pm_substates_capabilities: caps,
            l1_pm_substates_control_1: ctl1,
            l1_pm_substates_control_2: ctl2,
        } = self.data;
        if (0u32, 0u32, 0u32) == (caps.clone().into(), ctl1.clone().into(), ctl2.clone().into()) {
            return writeln!(f, "\t\t<unreadable>");
        }
        writeln!(f,
            "\t\tL1SubCap: PCI-PM_L1.2{} PCI-PM_L1.1{} ASPM_L1.2{} ASPM_L1.1{} L1_PM_Substates{}",
            caps.pci_pm_l1_2_supported.display(BoolView::PlusMinus),
            caps.pci_pm_l1_1_supported.display(BoolView::PlusMinus),
            caps.aspm_l1_2_supported.display(BoolView::PlusMinus),
            caps.aspm_l1_1_supported.display(BoolView::PlusMinus),
            caps.l1_pm_substates_supported.display(BoolView::PlusMinus),
        )?;
        let is_l1_2_supported = caps.pci_pm_l1_2_supported || caps.aspm_l1_2_supported;
        if is_l1_2_supported {
            write!(f,
                "\t\t\t  PortCommonModeRestoreTime={}us ",
                caps.port_common_mode_restore_time
            )?;
            if let Some(time) = caps.port_t_power_on.value() {
                writeln!(f, "PortTPowerOnTime={}us", time)?;
            } else {
                writeln!(f, "PortTPowerOnTime=<error>")?;
            };
        }
        writeln!(f,
            "\t\tL1SubCtl1: PCI-PM_L1.2{} PCI-PM_L1.1{} ASPM_L1.2{} ASPM_L1.1{}",
            ctl1.pci_pm_l1_2_enable.display(BoolView::PlusMinus),
            ctl1.pci_pm_l1_1_enable.display(BoolView::PlusMinus),
            ctl1.aspm_l1_2_enable.display(BoolView::PlusMinus),
            ctl1.aspm_l1_1_enable.display(BoolView::PlusMinus),
        )?;
        if is_l1_2_supported {
            write!(f, "\t\t\t   T_CommonMode={}us", ctl1.common_mode_restore_time)?;
            if caps.aspm_l1_2_supported {
                if ctl1.ltr_l1_2_threshold.scale > 5 {
                    write!(f, " LTR1.2_Threshold=<error>")?;
                } else {
                    write!(f, " LTR1.2_Threshold={}ns", ctl1.ltr_l1_2_threshold.value())?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "\t\tL1SubCtl2:")?;
        if is_l1_2_supported {
            if let Some(time) = ctl2.t_power_on.value() {
                write!(f, " T_PwrOn={}us", time)?;
            } else {
                write!(f, " T_PwrOn=<error>")?;
            };
        }
        writeln!(f)
    }
}

// 001Fh Precision Time Measurement (PTM) 
impl<'a> DisplayMultiViewBasic<Verbose> for PrecisionTimeMeasurement {}
impl<'a> fmt::Display for MultiView<&'a PrecisionTimeMeasurement, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Precision Time Measurement")?;
        if verbose < 2 {
            return Ok(());
        }
        let PrecisionTimeMeasurement {
            ptm_capability: caps,
            ptm_control: ctrl,
        } = self.data;
        if u32::from(caps.clone()) == 0 && u32::from(ctrl.clone()) == 0 {
            return writeln!(f, "\t\t<unreadable>");
        }
        writeln!(f,
            "\t\tPTMCap: Requester:{} Responder:{} Root:{}",
            caps.ptm_requester_capable.display(BoolView::PlusMinus),
            caps.ptm_responder_capable.display(BoolView::PlusMinus),
            caps.ptm_root_capable.display(BoolView::PlusMinus),
        )?;
        write!(f, "\t\tPTMClockGranularity: ",)?;
        match caps.local_clock_granularity {
            0x00 => writeln!(f, "Unimplemented"),
            0xff => writeln!(f, "Greater than 254ns"),
               v => writeln!(f, "{}ns", v),
        }?;
        writeln!(f,
            "\t\tPTMControl: Enabled:{} RootSelected:{}",
            ctrl.ptm_enable.display(BoolView::PlusMinus),
            ctrl.root_select.display(BoolView::PlusMinus),
        )?;
        write!(f, "\t\tPTMEffectiveGranularity: ",)?;
        match ctrl.effective_granularity {
            0x00 => writeln!(f, "Unknown"),
            0xff => writeln!(f, "Greater than 254ns"),
               v => writeln!(f, "{}ns", v),
        }
    }
}

