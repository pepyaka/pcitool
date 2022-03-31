use core::fmt;
use core::cmp::Ordering;

use pcics::capabilities::{
    PciExpress,
    pci_express::{
        ActiveStatePowerManagement,
        Capabilities,
        CompletionTimeoutRanges,
        CompletionTimeoutValue,
        CompliancePresetOrDeEmphasis,
        CrosslinkResolution,
        DeEmphasis,
        Device,
        Device2,
        DeviceCapabilities,
        DeviceControl,
        DeviceStatus,
        DeviceType,
        DownstreamComponentPresence,
        EmergencyPowerReduction,
        EndpointL0sAcceptableLatency,
        EndpointL1AcceptableLatency,
        ExtendedTagFieldSupported,
        IndicatorControl,
        L0sExitLatency,
        L1ExitLatency,
        Link,
        Link2,
        LinkSpeed,
        LinkWidth,
        LnSystemCls,
        MaxSize,
        Obff,
        ObffEnable,
        Root,
        Slot,
        Slot2,
        SupportedLinkSpeedsVector,
        TphCompleter,
        TransmitMargin,
    }
};

use crate::device;
use crate::view::{DisplayMultiViewBasic,BoolView,MultiView};


const LATENCY_L0S: [&str; 8] = [
    "<64ns", "<128ns", "<256ns", "<512ns", "<1us", "<2us", "<4us", "unlimited"
];
const LATENCY_L1: [&str; 8] = [
    "<1us", "<2us", "<4us", "<8us", "<16us", "<32us", "<64us", "unlimited"
];



pub struct PciExpressView<'a> {
    pub verbose: usize,
    pub device: &'a device::Device,
}

impl<'a> DisplayMultiViewBasic<PciExpressView<'a>> for PciExpress {}
impl<'a> fmt::Display for MultiView<&'a PciExpress, PciExpressView<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.verbose;
        let PciExpress {
            capabilities: Capabilities {
                version,
                device_type,
                slot_implemented,
                interrupt_message_number,
                ..
            },
            device: Device {
                capabilities: DeviceCapabilities {
                    max_payload_size_supported,
                    phantom_functions_supported,
                    extended_tag_field_supported,
                    endpoint_l0s_acceptable_latency,
                    endpoint_l1_acceptable_latency,
                    attention_button_present,
                    attention_indicator_present,
                    power_indicator_present,
                    role_based_error_reporting,
                    function_level_reset_capability,
                    captured_slot_power_limit,
                },
                control: DeviceControl {
                    correctable_error_reporting_enable,
                    non_fatal_error_reporting_enable,
                    fatal_error_reporting_enable,
                    unsupported_request_reporting_enable,
                    enable_relaxed_ordering,
                    extended_tag_field_enable,
                    phantom_functions_enable,
                    aux_power_pm_enable,
                    enable_no_snoop,
                    bcre_or_flreset,
                    max_payload_size,
                    max_read_request_size,
                },
                status: DeviceStatus {
                    correctable_error_detected,
                    non_fatal_error_detected,
                    fatal_error_detected,
                    unsupported_request_detected,
                    aux_power_detected,
                    transactions_pending,
                },
            },
            ..
        } = self.data;
        write!(f, "Express ")?;
        if verbose > 1 {
            write!(f, "(v{}) ", version)?;
        }
        let has_slot = slot_implemented.display(BoolView::PlusMinus);
        match device_type {
            DeviceType::Endpoint => write!(f, "Endpoint"),
            DeviceType::LegacyEndpoint => write!(f, "Legacy Endpoint"),
            DeviceType::RootPort => write!(f, "Root Port (Slot{})", has_slot),
            DeviceType::UpstreamPort => write!(f, "Upstream Port"),
            DeviceType::DownstreamPort => write!(f, "Downstream Port (Slot{})", has_slot),
            DeviceType::PcieToPciBridge => write!(f, "PCI-Express to PCI/PCI-X Bridge"),
            DeviceType::PciToPcieBridge => write!(f, "PCI/PCI-X to PCI-Express Bridge (Slot{})", has_slot),
            DeviceType::RootComplexIntegratedEndpoint => write!(f, "Root Complex Integrated Endpoint"),
            DeviceType::RootComplexEventCollector => write!(f, "Root Complex Event Collector"),
            DeviceType::Reserved(id) => write!(f, "Unknown type {}", id),
        }?;
        writeln!(f, ", MSI {:02x}", interrupt_message_number)?;
        if verbose < 2 {
            return Ok(())
        }
        write!(f, 
            "\t\tDevCap:\tMaxPayload {} bytes, PhantFunc {}",
            max_payload_size_supported.display(()),
            *phantom_functions_supported as u8,
        )?;
        if let DeviceType::Endpoint | DeviceType::LegacyEndpoint = device_type {
            write!(f, 
                ", Latency L0s {}, L1 {}",
                endpoint_l0s_acceptable_latency.display(()),
                endpoint_l1_acceptable_latency.display(()),
            )?;
        }
        writeln!(f)?;
        write!(f, "\t\t\tExtTag{}", extended_tag_field_supported.display(()))?;
        if let DeviceType::Endpoint | DeviceType::LegacyEndpoint |
            DeviceType::UpstreamPort | DeviceType::PcieToPciBridge = device_type
        {
           write!(f,
               " AttnBtn{} AttnInd{} PwrInd{}",
               attention_button_present.display(BoolView::PlusMinus),
               attention_indicator_present.display(BoolView::PlusMinus),
               power_indicator_present.display(BoolView::PlusMinus),
           )?;
        }
        write!(f, " RBE{}", role_based_error_reporting.display(BoolView::PlusMinus))?;
        if let DeviceType::Endpoint | DeviceType::LegacyEndpoint |
            DeviceType::RootComplexIntegratedEndpoint = device_type
        {
            write!(f, " FLReset{}", function_level_reset_capability.display(BoolView::PlusMinus))?;
        }
        if let DeviceType::Endpoint | DeviceType::UpstreamPort |
            DeviceType::PcieToPciBridge = device_type
        {
            write!(f, " SlotPowerLimit {:.3}W", f32::from(captured_slot_power_limit))?;
        }
        writeln!(f)?;
        writeln!(f,
            "\t\tDevCtl:\tCorrErr{} NonFatalErr{} FatalErr{} UnsupReq{}",
            correctable_error_reporting_enable.display(BoolView::PlusMinus),
            non_fatal_error_reporting_enable.display(BoolView::PlusMinus),
            fatal_error_reporting_enable.display(BoolView::PlusMinus),
            unsupported_request_reporting_enable.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\t\tRlxdOrd{} ExtTag{} PhantFunc{} AuxPwr{} NoSnoop{}",
            enable_relaxed_ordering.display(BoolView::PlusMinus),
            extended_tag_field_enable.display(BoolView::PlusMinus),
            phantom_functions_enable.display(BoolView::PlusMinus),
            aux_power_pm_enable.display(BoolView::PlusMinus),
            enable_no_snoop.display(BoolView::PlusMinus),
        )?;
        if let DeviceType::PcieToPciBridge = device_type {
            write!(f, " BrConfRtry{}", bcre_or_flreset.display(BoolView::PlusMinus))?;
        }
        if let (
            DeviceType::Endpoint | DeviceType::LegacyEndpoint | DeviceType::RootComplexIntegratedEndpoint,
            true
        ) = (device_type, function_level_reset_capability) {
            write!(f, " FLReset{}", bcre_or_flreset.display(BoolView::PlusMinus))?;
        }
        write!(f,
            "\n\t\t\tMaxPayload {} bytes, MaxReadReq {} bytes\n",
            max_payload_size.display(()),
            max_read_request_size.display(()),
        )?;
        writeln!(f,
            "\t\tDevSta:\tCorrErr{} NonFatalErr{} FatalErr{} UnsupReq{} AuxPwr{} TransPend{}",
            correctable_error_detected.display(BoolView::PlusMinus),
            non_fatal_error_detected.display(BoolView::PlusMinus),
            fatal_error_detected.display(BoolView::PlusMinus),
            unsupported_request_detected.display(BoolView::PlusMinus),
            aux_power_detected.display(BoolView::PlusMinus),
            transactions_pending.display(BoolView::PlusMinus),
        )?;
        self.fmt_link(f)?;
        if self.data.capabilities.slot_implemented {
            self.fmt_slot(f)?;
        }
        if let DeviceType::RootPort | DeviceType::RootComplexEventCollector = device_type {
            self.fmt_root(f)?;
        }
        self.fmt_dev2(f)?;
        if self.data.link.is_some() {
            self.fmt_link2(f)?;
        }
        self.fmt_slot2(f)?;
        Ok(())
    }
}
impl<'a> MultiView<&'a PciExpress, PciExpressView<'a>> {
    fn fmt_link(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device_type = &self.data.capabilities.device_type;
        if let Some(Link { capabilities: caps, control: ctrl, status: st }) = &self.data.link {
            write!(f,
                "\t\tLnkCap:\tPort #{}, Speed {}, Width {}, ASPM {}",
                caps.port_number,
                caps.max_link_speed.display(SupportOnly2GTps(false)),
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
                caps.clock_power_management.display(BoolView::PlusMinus),
                caps.surprise_down_error_reporting_capable.display(BoolView::PlusMinus),
                caps.data_link_layer_link_active_reporting_capable.display(BoolView::PlusMinus),
                caps.link_bandwidth_notification_capability.display(BoolView::PlusMinus),
                caps.aspm_optionality_compliance.display(BoolView::PlusMinus),
            )?;
            write!(f, 
                "\t\tLnkCtl:\tASPM {};",
                ctrl.active_state_power_management_control.display(AspmView::Enabled)
            )?;
            if let DeviceType::RootPort | DeviceType::Endpoint |
                DeviceType::LegacyEndpoint | DeviceType::PcieToPciBridge = device_type
            {
                write!(f, " RCB {} bytes,", ctrl.read_completion_boundary as usize)?;
            }
            write!(f,
                " Disabled{} CommClk{}\n\t\t\tExtSynch{} ClockPM{} AutWidDis{} BWInt{} AutBWInt{}\n",
                ctrl.link_disable.display(BoolView::PlusMinus),
                ctrl.common_clock_configuration.display(BoolView::PlusMinus),
                ctrl.extended_synch.display(BoolView::PlusMinus),
                ctrl.enable_clock_power_management.display(BoolView::PlusMinus),
                ctrl.hardware_autonomous_width_disable.display(BoolView::PlusMinus),
                ctrl.link_bandwidth_management_interrupt_enable.display(BoolView::PlusMinus),
                ctrl.link_autonomous_bandwidth_interrupt_enable.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\tLnkSta:\tSpeed {} ({}), Width {} ({})",
                st.current_link_speed.display(SupportOnly2GTps(false)),
                match u8::from(st.current_link_speed).cmp(&u8::from(caps.max_link_speed)) {
                    Ordering::Less => "downgraded",
                    Ordering::Greater => "strange",
                    Ordering::Equal => "ok",
                },
                st.negotiated_link_width.display(()),
                match st.negotiated_link_width.cmp(&caps.maximum_link_width) {
                    Ordering::Less => "downgraded",
                    Ordering::Greater => "strange",
                    Ordering::Equal => "ok",
                },
            )?;
            writeln!(f,
                "\t\t\tTrErr{} Train{} SlotClk{} DLActive{} BWMgmt{} ABWMgmt{}",
                st.link_training_error.display(BoolView::PlusMinus),
                st.link_training.display(BoolView::PlusMinus),
                st.slot_clock_configuration.display(BoolView::PlusMinus),
                st.data_link_layer_link_active.display(BoolView::PlusMinus),
                st.link_bandwidth_management_status.display(BoolView::PlusMinus),
                st.link_autonomous_bandwidth_status.display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
    fn fmt_slot(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(Slot { capabilities: caps, control: ctrl, status: st }) = &self.data.slot {
            writeln!(f,
                "\t\tSltCap:\tAttnBtn{} PwrCtrl{} MRL{} AttnInd{} PwrInd{} HotPlug{} Surprise{}",
                caps.attention_button_present.display(BoolView::PlusMinus),
                caps.power_controller_present.display(BoolView::PlusMinus),
                caps.mrl_sensor_present.display(BoolView::PlusMinus),
                caps.attention_indicator_present.display(BoolView::PlusMinus),
                caps.power_indicator_present.display(BoolView::PlusMinus),
                caps.hot_plug_capable.display(BoolView::PlusMinus),
                caps.hot_plug_surprise.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\t\tSlot #{}, PowerLimit {:.3}W; Interlock{} NoCompl{}",
                caps.physical_slot_number,
                f32::from(&caps.slot_power_limit),
                caps.electromechanical_interlock_present.display(BoolView::PlusMinus),
                caps.no_command_completed_support.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\tSltCtl:\tEnable: AttnBtn{} PwrFlt{} MRL{} PresDet{} CmdCplt{} HPIrq{} LinkChg{}",
                ctrl.attention_button_pressed_enable.display(BoolView::PlusMinus),
                ctrl.power_fault_detected_enable.display(BoolView::PlusMinus),
                ctrl.mrl_sensor_changed_enable.display(BoolView::PlusMinus),
                ctrl.presence_detect_changed_enable.display(BoolView::PlusMinus),
                ctrl.command_completed_interrupt_enable.display(BoolView::PlusMinus),
                ctrl.hot_plug_interrupt_enable.display(BoolView::PlusMinus),
                ctrl.data_link_layer_state_changed_enable.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\t\tControl: AttnInd {}, PwrInd {}, Power{} Interlock{}",
                ctrl.attention_indicator_control.display(()),
                ctrl.power_indicator_control.display(()),
                ctrl.power_controller_control.display(BoolView::PlusMinus),
                ctrl.electromechanical_interlock_control.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\tSltSta:\tStatus: AttnBtn{} PowerFlt{} MRL{} CmdCplt{} PresDet{} Interlock{}",
                st.attention_button_pressed.display(BoolView::PlusMinus),
                st.power_fault_detected.display(BoolView::PlusMinus),
                st.mrl_sensor_state.display(BoolView::PlusMinus),
                st.command_completed.display(BoolView::PlusMinus),
                st.presence_detect_state.display(BoolView::PlusMinus),
                st.electromechanical_interlock_status.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\t\tChanged: MRL{} PresDet{} LinkState{}",
                st.mrl_sensor_changed.display(BoolView::PlusMinus),
                st.presence_detect_changed.display(BoolView::PlusMinus),
                st.data_link_layer_state_changed.display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
    fn fmt_root(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(Root { capabilities: caps, control: ctrl, status: st }) = &self.data.root {
            writeln!(f,
                "\t\tRootCap: CRSVisible{}",
                caps.crs_software_visibility.display(BoolView::PlusMinus)
            )?;
            writeln!(f,
                "\t\tRootCtl: ErrCorrectable{} ErrNon-Fatal{} ErrFatal{} PMEIntEna{} CRSVisible{}",
                ctrl.system_error_on_correctable_error_enable.display(BoolView::PlusMinus),
                ctrl.system_error_on_non_fatal_error_enable.display(BoolView::PlusMinus),
                ctrl.system_error_on_fatal_error_enable.display(BoolView::PlusMinus),
                ctrl.pme_interrupt_enable.display(BoolView::PlusMinus),
                ctrl.crs_software_visibility_enable.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\tRootSta: PME ReqID {:04x}, PMEStatus{} PMEPending{}",
                st.pme_requester_id,
                st.pme_status.display(BoolView::PlusMinus),
                st.pme_pending.display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
    fn fmt_dev2(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device_type = &self.data.capabilities.device_type;

        // Device2 always printed in version > 1
        let zero_filled_device_2 = Device2::new(0, 0, 0);
        let device_2 = self.data.device_2.as_ref().or(Some(&zero_filled_device_2))
            .filter(|_| self.data.capabilities.version > 1);

        if let Some(Device2 { capabilities: caps, control: ctrl, status: _st }) = device_2 {
            write!(f,
                "\t\tDevCap2: Completion Timeout: {}, TimeoutDis{} NROPrPrP{} LTR{}",
                caps.completion_timeout_ranges_supported.display(()),
                caps.completion_timeout_disable_supported.display(BoolView::PlusMinus),
                caps.no_ro_enabled_pr_pr_passing.display(BoolView::PlusMinus),
                caps.ltr_mechanism_supported.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\n\t\t\t 10BitTagComp{} 10BitTagReq{} OBFF {}, ExtFmt{} EETLPPrefix{}",
                caps.support_10bit_tag_completer.display(BoolView::PlusMinus),
                caps.support_10bit_tag_requester.display(BoolView::PlusMinus),
                caps.obff_supported.display(()),
                caps.extended_fmt_field_supported.display(BoolView::PlusMinus),
                caps.end_end_tlp_prefix_supported.display(BoolView::PlusMinus),
            )?;
            if caps.end_end_tlp_prefix_supported {
                let meetlp = caps.max_end_end_tlp_prefixes as u8;
                let meetlp = if meetlp == 0 { 4 } else { meetlp };
                write!(f, ", MaxEETLPPrefixes {}", meetlp)?;
            }
            write!(f,
                "\n\t\t\t EmergencyPowerReduction {}, EmergencyPowerReductionInit{}",
                caps.emergency_power_reduction_supported.display(()),
                caps.emergency_power_reduction_initialization_required.display(BoolView::PlusMinus),
            )?;
            write!(f, "\n\t\t\t FRS{}", caps.frs_supported.display(BoolView::PlusMinus))?;
            if let DeviceType::RootPort = device_type {
                write!(f," LN System CLS {},", caps.ln_system_cls.display(()))?;
            }
            if let DeviceType::RootPort | DeviceType::Endpoint = device_type {
                write!(f," {}", caps.tph_completer_supported.display(()))?;
            }
            if let DeviceType::RootPort | DeviceType::DownstreamPort = device_type {
                write!(f," ARIFwd{}", caps.ari_forwarding_supported.display(BoolView::PlusMinus))?;
            }
            writeln!(f)?;
            let has_mem_bar = self.view.device.has_mem_bar();
            let is_rp_up_dp =
                matches!(device_type, DeviceType::RootPort | DeviceType::UpstreamPort | DeviceType::DownstreamPort);
            if is_rp_up_dp || has_mem_bar {
                write!(f,"\t\t\t AtomicOpsCap:")?;
                if is_rp_up_dp {
                    write!(f," Routing{}", caps.atomic_op_routing_supported.display(BoolView::PlusMinus))?;
                }
                if matches!(device_type, DeviceType::RootPort) || has_mem_bar {
                    write!(f,
                        " 32bit{} 64bit{} 128bitCAS{}",
                        caps.u32_atomicop_completer_supported.display(BoolView::PlusMinus),
                        caps.u64_atomicop_completer_supported.display(BoolView::PlusMinus),
                        caps.u128_cas_completer_supported.display(BoolView::PlusMinus),
                    )?;
                }
                writeln!(f)?;
            }
            write!(f,
                // "\t\tDevCtl2: Completion Timeout: {}, TimeoutDis{} LTR{} 10BitTagReq{} OBFF {},",
                "\t\tDevCtl2: Completion Timeout: {}, TimeoutDis{} LTR{} OBFF {},",
                ctrl.completion_timeout_value.display(()),
                ctrl.completion_timeout_disable.display(BoolView::PlusMinus),
                ctrl.ltr_mechanism_enable.display(BoolView::PlusMinus),
                // ctrl.enable_10bit_tag_requester.display(BoolView::PlusMinus),
                ctrl.obff_enable.display(()),
            )?;
            if matches!(device_type, DeviceType::RootPort | DeviceType::DownstreamPort) {
                write!(f," ARIFwd{}", ctrl.ari_forwarding_enable.display(BoolView::PlusMinus))?;
            }
            writeln!(f)?;
            if matches!(device_type,
                DeviceType::RootPort | DeviceType::UpstreamPort |
                DeviceType::DownstreamPort | DeviceType::Endpoint |
                DeviceType::RootComplexIntegratedEndpoint | DeviceType::LegacyEndpoint
            ) {
                write!(f, "\t\t\t AtomicOpsCtl:")?;
                if matches!(device_type,
                    DeviceType::RootPort | DeviceType::Endpoint |
                    DeviceType::RootComplexIntegratedEndpoint | DeviceType::LegacyEndpoint
                ) {
                    write!(f, " ReqEn{}", ctrl.atomic_op_requester_enable.display(BoolView::PlusMinus))?;
                }
                if matches!(device_type,
                    DeviceType::RootPort | DeviceType::UpstreamPort | DeviceType::DownstreamPort
                ) {
                    write!(f, " EgressBlck{}", ctrl.atomic_op_egress_blocking.display(BoolView::PlusMinus))?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
    fn fmt_link2(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let zero_filled_link_2 = Link2 {
            capabilities: 0.into(),
            control: 0.into(),
            status: 0.into(),
        };
        if let Some(Link2 {
            capabilities: caps,
            control: ctrl,
            status: st
        }) = self.data.link_2.as_ref().or(Some(&zero_filled_link_2)) {
            let device_type = &self.data.capabilities.device_type;
            if !(matches!(device_type, DeviceType::Endpoint | DeviceType::LegacyEndpoint) &&
                (self.view.device.address.device != 0 || self.view.device.address.function != 0))
            {
                if u32::from(caps.clone()) != 0 {
                    writeln!(f,
                        "\t\tLnkCap2: Supported Link Speeds: {}, Crosslink{} Retimer{} 2Retimers{} DRS{}",
                        caps.supported_link_speeds_vector.display(()),
                        caps.crosslink_supported.display(BoolView::PlusMinus),
                        caps.retimer_presence_detect_supported.display(BoolView::PlusMinus),
                        caps.two_retimers_presence_detect_supported.display(BoolView::PlusMinus),
                        caps.drs_supported.display(BoolView::PlusMinus),
                    )?;
                }
                write!(f,
                    "\t\tLnkCtl2: Target Link Speed: {}, EnterCompliance{} SpeedDis{}",
                    ctrl.target_link_speed.display(SupportOnly2GTps(true)),
                    ctrl.enter_compliance.display(BoolView::PlusMinus),
                    ctrl.hardware_autonomous_speed_disable.display(BoolView::PlusMinus),
                )?;
                if matches!(device_type, DeviceType::DownstreamPort) {
                    write!(f, ", Selectable De-emphasis: {}", ctrl.selectable_de_emphasis.display(()))?;
                }
                write!(f,
                    "\n\t\t\t Transmit Margin: {}, EnterModifiedCompliance{} ComplianceSOS{}",
                    ctrl.transmit_margin.display(()),
                    ctrl.enter_modified_compliance.display(BoolView::PlusMinus),
                    ctrl.compliance_sos.display(BoolView::PlusMinus),
                )?;
                write!(f,
                    "\n\t\t\t Compliance De-emphasis: {}\n",
                    ctrl.compliance_preset_or_de_emphasis.display(LinkSpeed::Rate5GTps)
                )?;
            }
            writeln!(f,
                "\t\tLnkSta2: Current De-emphasis Level: {}, EqualizationComplete{} EqualizationPhase1{}",
                st.current_de_emphasis_level.display(()),
                st.equalization_complete.display(BoolView::PlusMinus),
                st.equalization_phase_1_successful.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\t\t EqualizationPhase2{} EqualizationPhase3{} LinkEqualizationRequest{}",
                st.equalization_phase_2_successful.display(BoolView::PlusMinus),
                st.equalization_phase_3_successful.display(BoolView::PlusMinus),
                st.link_equalization_request.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\t\t Retimer{} 2Retimers{} CrosslinkRes: {}",
                st.retimer_presence_detected.display(BoolView::PlusMinus),
                st.two_retimers_presence_detected.display(BoolView::PlusMinus),
                st.crosslink_resolution.display(()),
            )?;
            if matches!(device_type, DeviceType::RootPort | DeviceType::DownstreamPort | DeviceType::PcieToPciBridge )
                && caps.drs_supported
            {
                write!(f,
                    ", DRS{}\n\t\t\t DownstreamComp: {}",
                    st.drs_message_received.display(BoolView::PlusMinus),
                    st.downstream_component_presence.display(()),
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
    fn fmt_slot2(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(Slot2 { capabilities: _caps, control: _ctrl, status: _st }) = &self.data.slot_2 {
            // There is no output in lspci
        }
        Ok(())
    }
}


impl DisplayMultiViewBasic<()> for ExtendedTagFieldSupported {}
impl<'a> fmt::Display for MultiView<&'a ExtendedTagFieldSupported, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ExtendedTagFieldSupported::Five => write!(f, "-"),
            ExtendedTagFieldSupported::Eight => write!(f, "+"),
        }
    }
}

impl DisplayMultiViewBasic<()> for MaxSize {}
impl<'a> fmt::Display for MultiView<&'a MaxSize, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size: usize = 128 << (*self.data as u8);
        write!(f, "{}", size)
    }
}

/// Components that support only the 2.5 GT/s speed are permitted to hardwire [LinkSpeed] to 0000b.
pub struct SupportOnly2GTps(pub bool);

impl DisplayMultiViewBasic<SupportOnly2GTps> for LinkSpeed {}
impl fmt::Display for MultiView<&LinkSpeed, SupportOnly2GTps> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.view.0, self.data) {
            (true, LinkSpeed::Reserved(0)) => write!(f, "2.5GT/s"),
            (_, LinkSpeed::Rate2GTps)  => write!(f, "2.5GT/s"),
            (_, LinkSpeed::Rate5GTps)  => write!(f, "5GT/s"),
            (_, LinkSpeed::Rate8GTps)  => write!(f, "8GT/s"),
            (_, LinkSpeed::Rate16GTps) => write!(f, "16GT/s"),
            (_, LinkSpeed::Rate32GTps) => write!(f, "32GT/s"),
            (_, LinkSpeed::Rate64GTps) => write!(f, "64GT/s"),
            _ => write!(f, "unknown"),
        }
    }
}

impl DisplayMultiViewBasic<()> for LinkWidth {}
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
            _ => write!(f, "unknown"),
        }
    }
}

enum AspmView {
    Support,
    Enabled,
}

impl DisplayMultiViewBasic<AspmView> for ActiveStatePowerManagement {}
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

impl DisplayMultiViewBasic<()> for EndpointL0sAcceptableLatency {}
impl fmt::Display for MultiView<&EndpointL0sAcceptableLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L0S[(*self.data as usize)])
    }
}

impl DisplayMultiViewBasic<()> for EndpointL1AcceptableLatency {}
impl fmt::Display for MultiView<&EndpointL1AcceptableLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L1[(*self.data as usize)])
    }
}

impl DisplayMultiViewBasic<()> for L0sExitLatency {}
impl fmt::Display for MultiView<&L0sExitLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L0S[(*self.data as usize)])
    }
}

impl DisplayMultiViewBasic<()> for L1ExitLatency {}
impl fmt::Display for MultiView<&L1ExitLatency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LATENCY_L1[(*self.data as usize)])
    }
}

impl DisplayMultiViewBasic<()> for IndicatorControl {}
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

impl DisplayMultiViewBasic<()> for CompletionTimeoutRanges {}
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
            CompletionTimeoutRanges::Reserved(_)  => "",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiViewBasic<()> for LnSystemCls {}
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

impl DisplayMultiViewBasic<()> for EmergencyPowerReduction {}
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

impl DisplayMultiViewBasic<()> for Obff {}
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

impl DisplayMultiViewBasic<()> for TphCompleter {}
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

impl DisplayMultiViewBasic<()> for CompletionTimeoutValue {}
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

impl DisplayMultiViewBasic<()> for ObffEnable {}
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

impl DisplayMultiViewBasic<()> for DeEmphasis {}
impl fmt::Display for MultiView<&DeEmphasis, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            DeEmphasis::Minus3_5dB => "-3.5dB",
            DeEmphasis::Minus6dB => "-6dB",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiViewBasic<()> for TransmitMargin {}
impl fmt::Display for MultiView<&TransmitMargin, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data.0 {
            0 => "Normal Operating Range",
            1 => "800-1200mV(full-swing)/400-700mV(half-swing)",
            2..=4 => "",
            5 => "200-400mV(full-swing)/100-200mV(half-swing)",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiViewBasic<LinkSpeed> for CompliancePresetOrDeEmphasis {}
impl fmt::Display for MultiView<&CompliancePresetOrDeEmphasis, LinkSpeed> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match (&self.view, self.data.0) {
            (LinkSpeed::Rate2GTps, _) => "",
            (LinkSpeed::Rate5GTps, 0b000) => "-6dB",
            (LinkSpeed::Rate5GTps, 0b001) => "-3.5dB",
            _ => "",
        };
        write!(f, "{}", s)
    }
}

impl DisplayMultiViewBasic<()> for CrosslinkResolution {}
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

impl DisplayMultiViewBasic<()> for DownstreamComponentPresence {}
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

impl DisplayMultiViewBasic<()> for SupportedLinkSpeedsVector {}
impl fmt::Display for MultiView<&SupportedLinkSpeedsVector, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SupportedLinkSpeedsVector {
            speed_2_5_gtps: s2,
            speed_5_0_gtps: s5,
            speed_8_0_gtps: s8,
            speed_16_0_gtps: s16,
            speed_32_0_gtps: s32,
            speed_64_0_gtps: s64,
        } = *self.data;
        let s = match (s64, s32, s16, s8, s5, s2) {
            (true, ..) => "2.5-64GT/s",
            (false, true, ..) => "2.5-32GT/s",
            (false, false, true, ..) => "2.5-16GT/s",
            (false, false, false, true, ..) => "2.5-8GT/s",
            (false, false, false, false, true, ..) => "2.5-5GT/s",
            (false, false, false, false, false, true) => "2.5GT/s",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}
