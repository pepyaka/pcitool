use core::fmt;
use core::cmp::Ordering;

use pcics::DDR_OFFSET;
use pcics::capabilities::pci_express::{
    PciExpress, ActiveStatePowerManagement, CompletionTimeoutRanges, CompletionTimeoutValue,
    CompliancePresetOrDeEmphasis, CrosslinkResolution, DeEmphasis, Device, Device2,
    DeviceType, DownstreamComponentPresence,
    EmergencyPowerReduction, EndpointL0sAcceptableLatency, EndpointL1AcceptableLatency,
    ExtendedTagFieldSupported, IndicatorControl, L0sExitLatency, L1ExitLatency, Link, Link2,
    LinkSpeed, LinkWidth, LnSystemCls, MaxSize, Obff, ObffEnable, Root, Slot, Slot2,
    SupportedLinkSpeedsVector, TphCompleter, TransmitMargin, 
};

use crate::device;
use crate::view::lspci::basic::Flag;
use crate::view::{DisplayMultiView,MultiView};


const LATENCY_L0S: [&str; 8] = [
    "<64ns", "<128ns", "<256ns", "<512ns", "<1us", "<2us", "<4us", "unlimited"
];
const LATENCY_L1: [&str; 8] = [
    "<1us", "<2us", "<4us", "<8us", "<16us", "<32us", "<64us", "unlimited"
];



pub struct PciExpressView<'a> {
    pub pointer: u8,
    pub verbose: usize,
    pub device: &'a device::Device,
}

impl<'a> DisplayMultiView<PciExpressView<'a>> for PciExpress {}
impl<'a> fmt::Display for MultiView<&'a PciExpress, PciExpressView<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.verbose;
        let PciExpress {
            version,
            device_type,
            slot_implemented,
            interrupt_message_number,
            device,
            device_2,
            ..
        } = &self.data;
        write!(f, "Express ")?;
        if verbose > 1 {
            write!(f, "(v{}) ", version)?;
        }
        let slot_view = Flag(*slot_implemented);
        let (link, slot, root, link_2, slot_2) = 
            match device_type {
                DeviceType::Endpoint { link, link_2  } => {
                    write!(f, "Endpoint")?;
                    (Some(link), None, None, link_2.as_ref(), None)
                },
                DeviceType::LegacyEndpoint { link, link_2 } => {
                    write!(f, "Legacy Endpoint")?;
                    (Some(link), None, None, link_2.as_ref(), None)
                },
                DeviceType::RootPort { link, link_2, slot, slot_2, root } => {
                    write!(f, "Root Port (Slot{})", slot_view)?;
                    (Some(link), Some(slot), Some(root), link_2.as_ref(), slot_2.as_ref())
                },
                DeviceType::UpstreamPort { link, link_2 } => {
                    write!(f, "Upstream Port")?;
                    (Some(link), None, None, link_2.as_ref(), None)
                },
                DeviceType::DownstreamPort { link, link_2, slot, slot_2 } => {
                    write!(f, "Downstream Port (Slot{})", slot_view)?;
                    (Some(link), Some(slot), None, link_2.as_ref(), slot_2.as_ref())
                },
                DeviceType::PcieToPciBridge { link, link_2 } => {
                    write!(f, "PCI-Express to PCI/PCI-X Bridge")?;
                    (Some(link), None, None, link_2.as_ref(), None)
                },
                DeviceType::PciToPcieBridge { link, link_2, slot, slot_2 } => {
                    write!(f, "PCI/PCI-X to PCI-Express Bridge (Slot{})", slot_view)?;
                    (Some(link), Some(slot), None, link_2.as_ref(), slot_2.as_ref())
                },
                DeviceType::RootComplexIntegratedEndpoint => {
                    write!(f, "Root Complex Integrated Endpoint")?;
                    (None, None, None, None, None)
                },
                DeviceType::RootComplexEventCollector { root } => {
                    write!(f, "Root Complex Event Collector")?;
                    (None, None, Some(root), None, None)
                },
                DeviceType::Reserved { id, link, link_2, .. } => {
                    write!(f, "Unknown type {}", id)?;
                    (Some(link), None, None, link_2.as_ref(), None)
                },
            };
        writeln!(f, ", MSI {:02x}", interrupt_message_number)?;
        if verbose < 2 {
            return Ok(())
        }
        self.fmt_device(f, device)?;
        if let Some(link) = link {
            self.fmt_link(f, link)?;
        }
        if let (true, Some(slot)) = (slot_implemented, slot) {
            self.fmt_slot(f, slot)?;
        }
        if let Some(root) = root {
            self.fmt_root(f, root)?;
        }
        if let Some(device_2) = device_2 {
            self.fmt_device_2(f, device_2)?;
        }
        if let (Some(_), Some(link_2)) = (link, link_2) {
            self.fmt_link_2(f, link_2)?;
        }
        if let (true, Some(_), Some(slot_2)) = (slot_implemented, slot, slot_2) {
            self.fmt_slot_2(f, slot_2)?;
        }
        Ok(())
    }
}
impl<'a> MultiView<&'a PciExpress, PciExpressView<'a>> {
    fn fmt_device(&self, f: &mut fmt::Formatter<'_>, device: &'a Device) -> fmt::Result {
        let PciExpress { device_type, .. } = &self.data;
        let Device { capabilities: caps, control: ctrl, status: st, } = device;
        write!(f, 
            "\t\tDevCap:\tMaxPayload {} bytes, PhantFunc {}",
            caps.max_payload_size_supported.display(()),
            (1 << (caps.phantom_functions_supported as u8)) - 1,
        )?;
        if let DeviceType::Endpoint { .. } | DeviceType::LegacyEndpoint { .. } = device_type {
            write!(f, 
                ", Latency L0s {}, L1 {}",
                caps.endpoint_l0s_acceptable_latency.display(()),
                caps.endpoint_l1_acceptable_latency.display(()),
            )?;
        }
        writeln!(f)?;
        write!(f, "\t\t\tExtTag{}", caps.extended_tag_field_supported.display(()))?;
        if let DeviceType::Endpoint { .. } | DeviceType::LegacyEndpoint { .. } |
            DeviceType::UpstreamPort { .. } | DeviceType::PcieToPciBridge  { .. } = device_type
        {
           write!(f,
               " AttnBtn{} AttnInd{} PwrInd{}",
               Flag(caps.attention_button_present),
               Flag(caps.attention_indicator_present),
               Flag(caps.power_indicator_present),
           )?;
        }
        write!(f, " RBE{}", Flag(caps.role_based_error_reporting))?;
        if let DeviceType::Endpoint { .. } | DeviceType::LegacyEndpoint { .. } |
            DeviceType::RootComplexIntegratedEndpoint = device_type
        {
            write!(f, " FLReset{}", Flag(caps.function_level_reset_capability))?;
        }
        if let DeviceType::Endpoint { .. } | DeviceType::UpstreamPort { .. } |
            DeviceType::PcieToPciBridge { .. } = device_type
        {
            write!(f, " SlotPowerLimit {:.3}W", f32::from(caps.captured_slot_power_limit.clone()))?;
        }
        writeln!(f)?;
        writeln!(f,
            "\t\tDevCtl:\tCorrErr{} NonFatalErr{} FatalErr{} UnsupReq{}",
            Flag(ctrl.correctable_error_reporting_enable),
            Flag(ctrl.non_fatal_error_reporting_enable),
            Flag(ctrl.fatal_error_reporting_enable),
            Flag(ctrl.unsupported_request_reporting_enable),
        )?;
        write!(f,
            "\t\t\tRlxdOrd{} ExtTag{} PhantFunc{} AuxPwr{} NoSnoop{}",
            Flag(ctrl.enable_relaxed_ordering),
            Flag(ctrl.extended_tag_field_enable),
            Flag(ctrl.phantom_functions_enable),
            Flag(ctrl.aux_power_pm_enable),
            Flag(ctrl.enable_no_snoop),
        )?;
        if let DeviceType::PcieToPciBridge { .. } = device_type {
            write!(f, " BrConfRtry{}", Flag(ctrl.bcre_or_flreset))?;
        }
        if let (
            DeviceType::Endpoint { .. } | DeviceType::LegacyEndpoint { .. } | DeviceType::RootComplexIntegratedEndpoint { .. },
            true
        ) = (device_type, caps.function_level_reset_capability) {
            write!(f, " FLReset{}", Flag(ctrl.bcre_or_flreset))?;
        }
        write!(f,
            "\n\t\t\tMaxPayload {} bytes, MaxReadReq {} bytes\n",
            ctrl.max_payload_size.display(()),
            ctrl.max_read_request_size.display(()),
        )?;
        writeln!(f,
            "\t\tDevSta:\tCorrErr{} NonFatalErr{} FatalErr{} UnsupReq{} AuxPwr{} TransPend{}",
            Flag(st.correctable_error_detected),
            Flag(st.non_fatal_error_detected),
            Flag(st.fatal_error_detected),
            Flag(st.unsupported_request_detected),
            Flag(st.aux_power_detected),
            Flag(st.transactions_pending),
        )
    }
    fn fmt_link(&self, f: &mut fmt::Formatter<'_>, link: &'a Link) -> fmt::Result {
        let device_type = &self.data.device_type;
        let Link { capabilities: caps, control: ctrl, status: st, } = link;
        write!(f,
            "\t\tLnkCap:\tPort #{}, Speed {}, Width {}, ASPM {}",
            caps.port_number,
            caps.max_link_speed.display(()),
            caps.maximum_link_width.display(()),
            caps.active_state_power_management_support.display(AspmView::Support),
        )?;
        match caps.active_state_power_management_support {
            ActiveStatePowerManagement::L0s =>
                write!(f, ", Exit Latency L0s {}", caps.l0s_exit_latency.display(()))?,
            ActiveStatePowerManagement::L1 =>
                write!(f, ", Exit Latency L1 {}", caps.l1_exit_latency.display(()))?,
            ActiveStatePowerManagement::L0sAndL1 =>
                write!(f,
                    ", Exit Latency L0s {}, L1 {}",
                    caps.l0s_exit_latency.display(()),
                    caps.l1_exit_latency.display(())
                )?,
            _ => (),
        }
        write!(f,
            "\n\t\t\tClockPM{} Surprise{} LLActRep{} BwNot{} ASPMOptComp{}\n",
            Flag(caps.clock_power_management),
            Flag(caps.surprise_down_error_reporting_capable),
            Flag(caps.data_link_layer_link_active_reporting_capable),
            Flag(caps.link_bandwidth_notification_capability),
            Flag(caps.aspm_optionality_compliance),
        )?;
        write!(f, 
            "\t\tLnkCtl:\tASPM {};",
            ctrl.active_state_power_management_control.display(AspmView::Enabled)
        )?;
        if let DeviceType::RootPort { .. } | DeviceType::Endpoint { .. } |
            DeviceType::LegacyEndpoint { .. } | DeviceType::PcieToPciBridge { .. } = device_type
        {
            write!(f, " RCB {} bytes,", ctrl.read_completion_boundary as usize)?;
        }
        write!(f,
            " Disabled{} CommClk{}\n\t\t\tExtSynch{} ClockPM{} AutWidDis{} BWInt{} AutBWInt{}\n",
            Flag(ctrl.link_disable),
            Flag(ctrl.common_clock_configuration),
            Flag(ctrl.extended_synch),
            Flag(ctrl.enable_clock_power_management),
            Flag(ctrl.hardware_autonomous_width_disable),
            Flag(ctrl.link_bandwidth_management_interrupt_enable),
            Flag(ctrl.link_autonomous_bandwidth_interrupt_enable),
        )?;
        // write!(f, "{} {}", u8::from(st.negotiated_link_width.clone()), u8::from(caps.maximum_link_width.clone()))?;
        writeln!(f,
            "\t\tLnkSta:\tSpeed {} ({}), Width {} ({})",
            st.current_link_speed.display(()),
            link_compare(u8::from(st.current_link_speed), u8::from(caps.max_link_speed)),
            st.negotiated_link_width.display(()),
            link_compare(
                u8::from(st.negotiated_link_width.clone()),
                u8::from(caps.maximum_link_width.clone())
            ),
        )?;
        writeln!(f,
            "\t\t\tTrErr{} Train{} SlotClk{} DLActive{} BWMgmt{} ABWMgmt{}",
            Flag(st.link_training_error),
            Flag(st.link_training),
            Flag(st.slot_clock_configuration),
            Flag(st.data_link_layer_link_active),
            Flag(st.link_bandwidth_management_status),
            Flag(st.link_autonomous_bandwidth_status),
        )?;
        Ok(())
    }
    fn fmt_slot(&self, f: &mut fmt::Formatter<'_>, slot: &'a Slot) -> fmt::Result {
        let Slot { capabilities: caps, control: ctrl, status: st, } = slot;
        writeln!(f,
            "\t\tSltCap:\tAttnBtn{} PwrCtrl{} MRL{} AttnInd{} PwrInd{} HotPlug{} Surprise{}",
            Flag(caps.attention_button_present),
            Flag(caps.power_controller_present),
            Flag(caps.mrl_sensor_present),
            Flag(caps.attention_indicator_present),
            Flag(caps.power_indicator_present),
            Flag(caps.hot_plug_capable),
            Flag(caps.hot_plug_surprise),
        )?;
        writeln!(f,
            "\t\t\tSlot #{}, PowerLimit {:.3}W; Interlock{} NoCompl{}",
            caps.physical_slot_number,
            f32::from(&caps.slot_power_limit),
            Flag(caps.electromechanical_interlock_present),
            Flag(caps.no_command_completed_support),
        )?;
        writeln!(f,
            "\t\tSltCtl:\tEnable: AttnBtn{} PwrFlt{} MRL{} PresDet{} CmdCplt{} HPIrq{} LinkChg{}",
            Flag(ctrl.attention_button_pressed_enable),
            Flag(ctrl.power_fault_detected_enable),
            Flag(ctrl.mrl_sensor_changed_enable),
            Flag(ctrl.presence_detect_changed_enable),
            Flag(ctrl.command_completed_interrupt_enable),
            Flag(ctrl.hot_plug_interrupt_enable),
            Flag(ctrl.data_link_layer_state_changed_enable),
        )?;
        writeln!(f,
            "\t\t\tControl: AttnInd {}, PwrInd {}, Power{} Interlock{}",
            ctrl.attention_indicator_control.display(()),
            ctrl.power_indicator_control.display(()),
            Flag(ctrl.power_controller_control),
            Flag(ctrl.electromechanical_interlock_control),
        )?;
        writeln!(f,
            "\t\tSltSta:\tStatus: AttnBtn{} PowerFlt{} MRL{} CmdCplt{} PresDet{} Interlock{}",
            Flag(st.attention_button_pressed),
            Flag(st.power_fault_detected),
            Flag(st.mrl_sensor_state),
            Flag(st.command_completed),
            Flag(st.presence_detect_state),
            Flag(st.electromechanical_interlock_status),
        )?;
        writeln!(f,
            "\t\t\tChanged: MRL{} PresDet{} LinkState{}",
            Flag(st.mrl_sensor_changed),
            Flag(st.presence_detect_changed),
            Flag(st.data_link_layer_state_changed),
        )?;
        Ok(())
    }
    fn fmt_root(&self, f: &mut fmt::Formatter<'_>, root: &'a Root) -> fmt::Result {
        let Root { capabilities: caps, control: ctrl, status: st, } = root;
        writeln!(f,
            "\t\tRootCap: CRSVisible{}",
            Flag(caps.crs_software_visibility)
        )?;
        writeln!(f,
            "\t\tRootCtl: ErrCorrectable{} ErrNon-Fatal{} ErrFatal{} PMEIntEna{} CRSVisible{}",
            Flag(ctrl.system_error_on_correctable_error_enable),
            Flag(ctrl.system_error_on_non_fatal_error_enable),
            Flag(ctrl.system_error_on_fatal_error_enable),
            Flag(ctrl.pme_interrupt_enable),
            Flag(ctrl.crs_software_visibility_enable),
        )?;
        writeln!(f,
            "\t\tRootSta: PME ReqID {:04x}, PMEStatus{} PMEPending{}",
            st.pme_requester_id,
            Flag(st.pme_status),
            Flag(st.pme_pending),
        )?;
        Ok(())
    }
    fn fmt_device_2(&self, f: &mut fmt::Formatter<'_>, device_2: &'a Device2) -> fmt::Result {
        let device_type = &self.data.device_type;
        let Device2 { capabilities: caps, control: ctrl, .. } = device_2;

        // // Device2 always printed in version > 1
        // let zero_filled_device_2 = Device2::new(0, 0, 0);
        // let device_2 = self.data.device_2.as_ref().or(Some(&zero_filled_device_2))
        //     .filter(|_| self.data.capabilities.version > 1);

        write!(f,
            "\t\tDevCap2: Completion Timeout: {}, TimeoutDis{} NROPrPrP{} LTR{}",
            caps.completion_timeout_ranges_supported.display(()),
            Flag(caps.completion_timeout_disable_supported),
            Flag(caps.no_ro_enabled_pr_pr_passing),
            Flag(caps.ltr_mechanism_supported),
        )?;
        write!(f,
            "\n\t\t\t 10BitTagComp{} 10BitTagReq{} OBFF {}, ExtFmt{} EETLPPrefix{}",
            Flag(caps.support_10bit_tag_completer),
            Flag(caps.support_10bit_tag_requester),
            caps.obff_supported.display(()),
            Flag(caps.extended_fmt_field_supported),
            Flag(caps.end_end_tlp_prefix_supported),
        )?;
        if caps.end_end_tlp_prefix_supported {
            let meetlp = caps.max_end_end_tlp_prefixes as u8;
            let meetlp = if meetlp == 0 { 4 } else { meetlp };
            write!(f, ", MaxEETLPPrefixes {}", meetlp)?;
        }
        write!(f,
            "\n\t\t\t EmergencyPowerReduction {}, EmergencyPowerReductionInit{}",
            caps.emergency_power_reduction_supported.display(()),
            Flag(caps.emergency_power_reduction_initialization_required),
        )?;
        write!(f, "\n\t\t\t FRS{}", Flag(caps.frs_supported))?;
        if let DeviceType::RootPort { .. } = device_type {
            write!(f," LN System CLS {},", caps.ln_system_cls.display(()))?;
        }
        if let DeviceType::RootPort { .. } | DeviceType::Endpoint { .. } = device_type {
            write!(f," {}", caps.tph_completer_supported.display(()))?;
        }
        if let DeviceType::RootPort { .. } | DeviceType::DownstreamPort { .. } = device_type {
            write!(f," ARIFwd{}", Flag(caps.ari_forwarding_supported))?;
        }
        writeln!(f)?;
        let has_mem_bar = self.view.device.has_mem_bar();
        let is_rp_up_dp =
            matches!(
                device_type, DeviceType::RootPort { .. } |
                DeviceType::UpstreamPort { .. } | DeviceType::DownstreamPort { .. }
            );
        if is_rp_up_dp || has_mem_bar {
            write!(f,"\t\t\t AtomicOpsCap:")?;
            if is_rp_up_dp {
                write!(f," Routing{}", Flag(caps.atomic_op_routing_supported))?;
            }
            if matches!(device_type, DeviceType::RootPort { .. }) || has_mem_bar {
                write!(f,
                    " 32bit{} 64bit{} 128bitCAS{}",
                    Flag(caps.u32_atomicop_completer_supported),
                    Flag(caps.u64_atomicop_completer_supported),
                    Flag(caps.u128_cas_completer_supported),
                )?;
            }
            writeln!(f)?;
        }
        write!(f,
            // "\t\tDevCtl2: Completion Timeout: {}, TimeoutDis{} LTR{} 10BitTagReq{} OBFF {},",
            "\t\tDevCtl2: Completion Timeout: {}, TimeoutDis{} LTR{} OBFF {},",
            ctrl.completion_timeout_value.display(()),
            Flag(ctrl.completion_timeout_disable),
            Flag(ctrl.ltr_mechanism_enable),
            // Flag(ctrl.enable_10bit_tag_requester),
            ctrl.obff_enable.display(()),
        )?;
        if matches!(device_type, DeviceType::RootPort { .. } | DeviceType::DownstreamPort { .. }) {
            write!(f," ARIFwd{}", Flag(ctrl.ari_forwarding_enable))?;
        }
        writeln!(f)?;
        if matches!(device_type,
            DeviceType::RootPort { .. } | DeviceType::UpstreamPort { .. } |
            DeviceType::DownstreamPort { .. } | DeviceType::Endpoint { .. } |
            DeviceType::RootComplexIntegratedEndpoint | DeviceType::LegacyEndpoint { .. }
        ) {
            write!(f, "\t\t\t AtomicOpsCtl:")?;
            if matches!(device_type,
                DeviceType::RootPort { .. } | DeviceType::Endpoint { .. } |
                DeviceType::RootComplexIntegratedEndpoint | DeviceType::LegacyEndpoint { .. }
            ) {
                write!(f, " ReqEn{}", Flag(ctrl.atomic_op_requester_enable))?;
            }
            if matches!(device_type,
                DeviceType::RootPort { .. } | DeviceType::UpstreamPort { .. } |
                DeviceType::DownstreamPort { .. }
            ) {
                write!(f, " EgressBlck{}", Flag(ctrl.atomic_op_egress_blocking))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
    fn fmt_link_2(&self, f: &mut fmt::Formatter<'_>, link_2: &'a Link2) -> fmt::Result {
        let Link2 { capabilities: caps, control: ctrl, status: st, } = link_2;
        let PciExpress { device_type, .. } = &self.data;
        let PciExpressView {
            pointer,
            device: device::Device { device_dependent_region, .. },
            ..
        } = &self.view;
        // let zero_filled_link_2 = Link2 {
        //     capabilities: 0.into(),
        //     control: 0.into(),
        //     status: 0.into(),
        // };
        if !(matches!(device_type, DeviceType::Endpoint { .. } | DeviceType::LegacyEndpoint { .. }) &&
            (self.view.device.address.device != 0 || self.view.device.address.function != 0))
        {
            let link_caps_2_offset = *pointer as usize - DDR_OFFSET + 0x2c;
            let is_empty_link_caps_2 =
                matches!(
                    device_dependent_region.as_ref()
                        .and_then(|ddr| ddr.0.get(link_caps_2_offset..(link_caps_2_offset + 2))),
                    Some([0, 0])
                );
            // if u32::from(caps.clone()) != 0 {
            if !is_empty_link_caps_2 {
                writeln!(f,
                    "\t\tLnkCap2: Supported Link Speeds: {}, Crosslink{} Retimer{} 2Retimers{} DRS{}",
                    caps.supported_link_speeds_vector.display(()),
                    Flag(caps.crosslink_supported),
                    Flag(caps.retimer_presence_detect_supported),
                    Flag(caps.two_retimers_presence_detect_supported),
                    Flag(caps.drs_supported),
                )?;
            }
            write!(f,
                "\t\tLnkCtl2: Target Link Speed: {}, EnterCompliance{} SpeedDis{}",
                ctrl.target_link_speed.display(SupportOnly2GTps).to_string()
                    .replace("unknown", "Unknown"),
                Flag(ctrl.enter_compliance),
                Flag(ctrl.hardware_autonomous_speed_disable),
            )?;
            if matches!(device_type, DeviceType::DownstreamPort { .. }) {
                write!(f, ", Selectable De-emphasis: {}", ctrl.selectable_de_emphasis.display(()))?;
            }
            write!(f,
                "\n\t\t\t Transmit Margin: {}, EnterModifiedCompliance{} ComplianceSOS{}",
                ctrl.transmit_margin.display(()),
                Flag(ctrl.enter_modified_compliance),
                Flag(ctrl.compliance_sos),
            )?;
            write!(f,
                "\n\t\t\t Compliance De-emphasis: {}\n",
                ctrl.compliance_preset_or_de_emphasis.display(LinkSpeed::Rate5GTps)
            )?;
        }
        writeln!(f,
            "\t\tLnkSta2: Current De-emphasis Level: {}, EqualizationComplete{} EqualizationPhase1{}",
            st.current_de_emphasis_level.display(()),
            Flag(st.equalization_complete),
            Flag(st.equalization_phase_1_successful),
        )?;
        writeln!(f,
            "\t\t\t EqualizationPhase2{} EqualizationPhase3{} LinkEqualizationRequest{}",
            Flag(st.equalization_phase_2_successful),
            Flag(st.equalization_phase_3_successful),
            Flag(st.link_equalization_request),
        )?;
        write!(f,
            "\t\t\t Retimer{} 2Retimers{} CrosslinkRes: {}",
            Flag(st.retimer_presence_detected),
            Flag(st.two_retimers_presence_detected),
            st.crosslink_resolution.display(()),
        )?;
        if device_type.is_downstream_port() && caps.drs_supported {
            write!(f,
                ", DRS{}\n\t\t\t DownstreamComp: {}",
                Flag(st.drs_message_received),
                st.downstream_component_presence.display(()),
            )?;
        }
        writeln!(f)?;
        Ok(())
    }
    fn fmt_slot_2(&self, _f: &mut fmt::Formatter<'_>, _slot_2: &'a Slot2) -> fmt::Result {
        // let Slot2 { capabilities: caps, control: ctrl, status: st, } = slot_2;
        // There is no output in lspci
        Ok(())
    }
}


impl DisplayMultiView<()> for ExtendedTagFieldSupported {}
impl<'a> fmt::Display for MultiView<&'a ExtendedTagFieldSupported, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ExtendedTagFieldSupported::Five => write!(f, "-"),
            ExtendedTagFieldSupported::Eight => write!(f, "+"),
        }
    }
}

impl DisplayMultiView<()> for MaxSize {}
impl<'a> fmt::Display for MultiView<&'a MaxSize, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size: usize = 128 << (*self.data as u8);
        write!(f, "{}", size)
    }
}

/// Components that support only the 2.5 GT/s speed are permitted to hardwire [LinkSpeed] to 0000b.
pub struct SupportOnly2GTps;

impl DisplayMultiView<()> for LinkSpeed {}
impl fmt::Display for MultiView<&LinkSpeed, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            LinkSpeed::Rate2GTps  => write!(f, "2.5GT/s"),
            LinkSpeed::Rate5GTps  => write!(f, "5GT/s"),
            LinkSpeed::Rate8GTps  => write!(f, "8GT/s"),
            LinkSpeed::Rate16GTps => write!(f, "16GT/s"),
            LinkSpeed::Rate32GTps => write!(f, "32GT/s"),
            // LinkSpeed::Rate64GTps => write!(f, "64GT/s"),
            _ => write!(f, "unknown"),
        }
    }
}
impl DisplayMultiView<SupportOnly2GTps> for LinkSpeed {}
impl fmt::Display for MultiView<&LinkSpeed, SupportOnly2GTps> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let LinkSpeed::Reserved(0) = self.data {
            write!(f, "2.5GT/s")
        } else {
            let view = MultiView { data: self.data, view: () };
            <MultiView<&LinkSpeed, ()>>::fmt(&view, f)
        }
    }
}

impl DisplayMultiView<()> for LinkWidth {}
impl fmt::Display for MultiView<&LinkWidth, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            LinkWidth::Reserved(0)  => write!(f, "x0"),
            LinkWidth::X1  => write!(f, "x1"),
            LinkWidth::X2  => write!(f, "x2"),
            LinkWidth::X4  => write!(f, "x4"),
            LinkWidth::X8  => write!(f, "x8"),
            LinkWidth::X12 => write!(f, "x12"),
            LinkWidth::X16 => write!(f, "x16"),
            LinkWidth::X32 => write!(f, "x32"),
            LinkWidth::Reserved(v) => write!(f, "x{}", v),
        }
    }
}

enum AspmView {
    Support,
    Enabled,
}

impl DisplayMultiView<AspmView> for ActiveStatePowerManagement {}
impl fmt::Display for MultiView<&ActiveStatePowerManagement, AspmView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ActiveStatePowerManagement::*;
        match (self.data, &self.view) {
            (NoAspm,   AspmView::Support) => write!(f, "not supported"),
            (L0s,      AspmView::Support) => write!(f, "L0s"),
            (L1,       AspmView::Support) => write!(f, "L1"),
            (L0sAndL1, AspmView::Support) => write!(f, "L0s L1"),
            (NoAspm,   AspmView::Enabled) => write!(f, "Disabled"),
            (L0s,      AspmView::Enabled) => write!(f, "L0s Enabled"),
            (L1,       AspmView::Enabled) => write!(f, "L1 Enabled"),
            (L0sAndL1, AspmView::Enabled) => write!(f, "L0s L1 Enabled"),
        }
    }
}

impl DisplayMultiView<()> for EndpointL0sAcceptableLatency {}
impl fmt::Display for MultiView<&EndpointL0sAcceptableLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L0S[(*self.data as usize)])
    }
}

impl DisplayMultiView<()> for EndpointL1AcceptableLatency {}
impl fmt::Display for MultiView<&EndpointL1AcceptableLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L1[(*self.data as usize)])
    }
}

impl DisplayMultiView<()> for L0sExitLatency {}
impl fmt::Display for MultiView<&L0sExitLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L0S[(*self.data as usize)])
    }
}

impl DisplayMultiView<()> for L1ExitLatency {}
impl fmt::Display for MultiView<&L1ExitLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L1[(*self.data as usize)])
    }
}

impl DisplayMultiView<()> for IndicatorControl {}
impl fmt::Display for MultiView<&IndicatorControl, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            IndicatorControl::Reserved => write!(f, "Unknown"),
            IndicatorControl::On       => write!(f, "On"),
            IndicatorControl::Blink    => write!(f, "Blink"),
            IndicatorControl::Off      => write!(f, "Off"),
        }
    }
}

impl DisplayMultiView<()> for CompletionTimeoutRanges {}
impl fmt::Display for MultiView<&CompletionTimeoutRanges, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            CompletionTimeoutRanges::NotSupported => "Not Supported",
            CompletionTimeoutRanges::RangeA       => "Range A",
            CompletionTimeoutRanges::RangeB       => "Range B",
            CompletionTimeoutRanges::RangesAB     => "Range AB",
            CompletionTimeoutRanges::RangesBC     => "Range BC",
            CompletionTimeoutRanges::RangesABC    => "Range ABC",
            CompletionTimeoutRanges::RangesBCD    => "Range BCD",
            CompletionTimeoutRanges::RangesABCD   => "Range ABCD",
            CompletionTimeoutRanges::Reserved(_)  => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for LnSystemCls {}
impl fmt::Display for MultiView<&LnSystemCls, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            LnSystemCls::NotSupported => "Not Supported",
            LnSystemCls::Cachelines64Byte => "64byte cachelines",
            LnSystemCls::Cachelines128Byte => "128byte cachelines",
            LnSystemCls::Reserved => "Reserved",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for EmergencyPowerReduction {}
impl fmt::Display for MultiView<&EmergencyPowerReduction, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            EmergencyPowerReduction::NotSupported => "Not Supported",
            EmergencyPowerReduction::DeviceSpecific => "Dev Specific",
            EmergencyPowerReduction::FormFactorOrDeviceSpecific => "Form Factor Dev Specific",
            EmergencyPowerReduction::Reserved => "Reserved",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for Obff {}
impl fmt::Display for MultiView<&Obff, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            Obff::NotSupported => "Not Supported",
            Obff::Message => "Via message",
            Obff::Wake => "Via WAKE#",
            Obff::WakeAndMessage => "Via message/WAKE#",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for TphCompleter {}
impl fmt::Display for MultiView<&TphCompleter, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            TphCompleter::NotSupported => "TPHComp- ExtTPHComp-",      
            TphCompleter::Tph => "TPHComp+ ExtTPHComp-",               
            TphCompleter::Reserved => "",          
            TphCompleter::TphAndExtendedTph => "TPHComp+ ExtTPHComp+", 
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for CompletionTimeoutValue {}
impl fmt::Display for MultiView<&CompletionTimeoutValue, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            CompletionTimeoutValue::DefaultRange50usTo50ms => "50us to 50ms",
            CompletionTimeoutValue::RangeA50usTo100us => "50us to 100us",
            CompletionTimeoutValue::RangeA1msTo10ms => "1ms to 10ms",
            CompletionTimeoutValue::RangeB16msTo55mss => "16ms to 55ms",
            CompletionTimeoutValue::RangeB65msTo210ms => "65ms to 210ms",
            CompletionTimeoutValue::RangeC260msTo900ms => "260ms to 900ms",
            CompletionTimeoutValue::RangeC1000msTo3500ms => "1s to 3.5s",
            CompletionTimeoutValue::RangeD4sTo13s => "4s to 13s",
            CompletionTimeoutValue::RangeD17sTo64s => "17s to 64s",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for ObffEnable {}
impl fmt::Display for MultiView<&ObffEnable, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            ObffEnable::Disabled => "Disabled",
            ObffEnable::MessageSignalingA => "Via message A",
            ObffEnable::MessageSignalingB => "Via message B",
            ObffEnable::WakeSignaling => "Via WAKE#",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for DeEmphasis {}
impl fmt::Display for MultiView<&DeEmphasis, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            DeEmphasis::Minus3_5dB => "-3.5dB",
            DeEmphasis::Minus6dB => "-6dB",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for TransmitMargin {}
impl fmt::Display for MultiView<&TransmitMargin, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data.0 {
            0 => "Normal Operating Range",
            1 => "800-1200mV(full-swing)/400-700mV(half-swing)",
            2..=5 => "200-400mV(full-swing)/100-200mV(half-swing)",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<LinkSpeed> for CompliancePresetOrDeEmphasis {}
impl fmt::Display for MultiView<&CompliancePresetOrDeEmphasis, LinkSpeed> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match (&self.view, self.data.0) {
            (LinkSpeed::Rate5GTps, 0b000) => "-6dB",
            (LinkSpeed::Rate5GTps, 0b001) => "-3.5dB",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for CrosslinkResolution {}
impl fmt::Display for MultiView<&CrosslinkResolution, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            CrosslinkResolution::NotSupported => "unsupported",
            CrosslinkResolution::UpstreamPort => "Upstream Port",
            CrosslinkResolution::DownstreamPort => "Downstream Port",
            CrosslinkResolution::NotCompleted => "incomplete",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for DownstreamComponentPresence {}
impl fmt::Display for MultiView<&DownstreamComponentPresence, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            DownstreamComponentPresence::DownNotDetermined => "Link Down - Not Determined",
            DownstreamComponentPresence::DownNotPresent => "Link Down - Not Present",
            DownstreamComponentPresence::DownPresent => "Link Down - Present",
            DownstreamComponentPresence::UpPresent => "Link Up - Present",
            DownstreamComponentPresence::UpPresentAndDrsReceived => "Link Up - Present and DRS Received",
            _ => "Reserved",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiView<()> for SupportedLinkSpeedsVector {}
impl fmt::Display for MultiView<&SupportedLinkSpeedsVector, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SupportedLinkSpeedsVector {
            speed_2_5_gtps: s2,
            speed_5_0_gtps: s5,
            speed_8_0_gtps: s8,
            speed_16_0_gtps: s16,
            speed_32_0_gtps: s32,
            speed_64_0_gtps: s64,
            reserved: rsvd,
        } = *self.data;
        let s = match (rsvd, s64, s32, s16, s8, s5, s2) {
            (_,  true, ..) => "RsvdP",
            (true,  ..) => "RsvdP",
            (false, false, true, ..) => "2.5-32GT/s",
            (false, false, false, true, ..) => "2.5-16GT/s",
            (false, false, false, false, true, ..) => "2.5-8GT/s",
            (false, false, false, false, false, true, ..) => "2.5-5GT/s",
            (false, false, false, false, false, false, true) => "2.5GT/s",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

fn link_compare<T: Ord>(sta: T, cap: T) -> &'static str {
    match sta.cmp(&cap) {
        Ordering::Less => "downgraded",
        Ordering::Greater => "strange",
        Ordering::Equal => "ok",
    }
}
