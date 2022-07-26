use core::fmt;

use pcics::{
    capabilities::PciExpress,
    extended_capabilities::{
        multifunction_virtual_channel::MultifunctionVirtualChannelError, tph_requester::StTable,
        AccessControlServices, AddressTranslationServices, AlternativeRoutingIdInterpretation,
        ConfigurationAccessCorrelation, DeviceSerialNumber, DownstreamPortContainment,
        ExtendedCapability, ExtendedCapabilityError, ExtendedCapabilityKind, L1PmSubstates,
        LatencyToleranceReporting, MultifunctionVirtualChannel, PageRequestInterface,
        PowerBudgeting, PrecisionTimeMeasurement, ProcessAddressSpaceId,
        RootComplexEventCollectorEndpointAssociation, RootComplexInternalLinkControl,
        RootComplexRegisterBlockHeader, SecondaryPciExpress, TphRequester,
        VendorSpecificExtendedCapability,
    },
};

use crate::{
    device::Device,
    view::{BoolView, DisplayMultiView, MultiView, Verbose},
};

use self::aer::AerView;
use self::vc::VcView;

use super::{BasicView, View};

pub struct EcapsView<'a> {
    pub view: &'a BasicView,
    pub device: &'a Device,
    pub maybe_pci_express: Option<&'a PciExpress>,
}

impl<'a> fmt::Display for View<(&'a ExtendedCapabilityError, &'a EcapsView<'a>)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (
            data,
            EcapsView {
                view: &BasicView { verbose, .. },
                ..
            },
        ) = self.0;
        let (offset, msg) = match data {
            offset @ ExtendedCapabilityError::Offset => (&0, format!("{}", offset)),
            header @ ExtendedCapabilityError::Header { offset } => (offset, format!("{}", header)),
            header @ ExtendedCapabilityError::EmptyHeader { offset } => {
                (offset, format!("{}", header))
            }
            ExtendedCapabilityError::Data { offset, source } => (offset, format!("{}", source)),
            ExtendedCapabilityError::AdvancedErrorReporting { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::RootComplexLinkDeclaration { offset, source } => {
                (offset, format!("{}", source.display(Verbose(verbose))))
            }
            ExtendedCapabilityError::SingleRootIoVirtualization { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::DownstreamPortContainment { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::MultifunctionVirtualChannel { offset, source } => {
                (offset, format!("{}", View(source)))
            }
        };
        let ver = if verbose > 1 { " v0" } else { "" };
        write!(f, "\tCapabilities: [{:03x}{}] {}", offset, ver, msg)
    }
}

impl<'a> DisplayMultiView<&'a EcapsView<'a>> for ExtendedCapability<'a> {}
impl<'a> fmt::Display for MultiView<&'a ExtendedCapability<'a>, &'a EcapsView<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let EcapsView {
            view,
            device,
            maybe_pci_express,
        } = self.view;
        let verbose = view.verbose;
        let offset = self.data.offset;
        write!(f, "\tCapabilities: [{:03x}", offset)?;
        if view.verbose > 1 {
            write!(f, " v{}", self.data.version)?;
        }
        write!(f, "] ")?;

        match &self.data.kind {
            // 0000h
            ExtendedCapabilityKind::Null => writeln!(f, "Null"),
            // 0001h
            ExtendedCapabilityKind::AdvancedErrorReporting(c) => {
                let is_type_root = maybe_pci_express
                    .filter(|pcie| pcie.device_type.is_root())
                    .is_some();
                write!(
                    f,
                    "{}",
                    c.display(AerView {
                        verbose,
                        is_type_root
                    })
                )
            }
            // 0002h
            ExtendedCapabilityKind::VirtualChannel(c) => {
                write!(f, "{}", c.display(VcView { verbose, offset }))
            }
            // 0003h
            ExtendedCapabilityKind::DeviceSerialNumber(c) => write!(f, "{}", c.display(())),
            // 0004h
            ExtendedCapabilityKind::PowerBudgeting(c) => write!(f, "{}", c.display(())),
            // 0005h
            ExtendedCapabilityKind::RootComplexLinkDeclaration(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 0006h
            ExtendedCapabilityKind::RootComplexInternalLinkControl(c) => {
                write!(f, "{}", View(c))
            }
            // 0007h
            ExtendedCapabilityKind::RootComplexEventCollectorEndpointAssociation(c) => {
                write!(f, "{}", View(c))
            }
            // 0008h
            ExtendedCapabilityKind::MultifunctionVirtualChannel(c) => {
                write!(f, "{}", View(c))
            }
            // 0009h
            ExtendedCapabilityKind::VirtualChannelMfvcPresent(c) => {
                write!(f, "{}", c.display(VcView { verbose, offset }))
            }
            // 000Ah
            ExtendedCapabilityKind::RootComplexRegisterBlockHeader(c) => {
                write!(f, "{}", View(c))
            }
            // 000Bh
            ExtendedCapabilityKind::VendorSpecificExtendedCapability(c) => {
                write!(f, "{}", c.display(()))
            }
            // 000Dh
            ExtendedCapabilityKind::AccessControlServices(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 000Eh
            ExtendedCapabilityKind::AlternativeRoutingIdInterpretation(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 000Fh
            ExtendedCapabilityKind::AddressTranslationServices(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 0010h
            ExtendedCapabilityKind::SingleRootIoVirtualization(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 0013h
            ExtendedCapabilityKind::PageRequestInterface(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 0017h
            ExtendedCapabilityKind::TphRequester(c) => write!(f, "{}", c.display(Verbose(verbose))),
            // 0018h
            ExtendedCapabilityKind::LatencyToleranceReporting(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 0019h
            ExtendedCapabilityKind::SecondaryPciExpress(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 001Bh
            ExtendedCapabilityKind::ProcessAddressSpaceId(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 001Dh
            ExtendedCapabilityKind::DownstreamPortContainment(c) => {
                // extended_configuration_space at 0x100
                let offset = offset as usize + 0x08 - 0x100;
                let maybe_raw_dpc_trigger_reason_extension = device
                    .extended_configuration_space
                    .as_ref()
                    .and_then(|ecs| ecs.0.get(offset).map(|byte| *byte >> 5 & 0b11));
                let view = DpcView {
                    verbose,
                    maybe_raw_dpc_trigger_reason_extension,
                };
                write!(f, "{}", c.display(view))
            }
            // 001Eh
            ExtendedCapabilityKind::L1PmSubstates(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            // 001Fh
            ExtendedCapabilityKind::PrecisionTimeMeasurement(c) => {
                write!(f, "{}", c.display(Verbose(verbose)))
            }
            ExtendedCapabilityKind::ConfigurationAccessCorrelation(_) => {
                writeln!(f, "Extended Capability ID {:#x}", &self.data.id())
            }
            _ => writeln!(f, "TODO {:?}", &self.data.kind),
        }
    }
}

// 0001h Advanced Error Reporting (AER)
mod aer;

// 0002h Virtual Channel (VC)
// 0009h Virtual Channel (VC) – used if an MFVC Extended Cap structure is present in the device
mod vc;

// 0003h Device Serial Number
impl<'a> DisplayMultiView<()> for DeviceSerialNumber {}
impl<'a> fmt::Display for MultiView<&'a DeviceSerialNumber, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DeviceSerialNumber {
            lower_dword,
            upper_dword,
        } = self.data;
        let [b0, b1, b2, b3] = lower_dword.to_le_bytes();
        let [b4, b5, b6, b7] = upper_dword.to_le_bytes();
        writeln!(
            f,
            "Device Serial Number {:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            b7, b6, b5, b4, b3, b2, b1, b0
        )
    }
}

// 0004h Power Budgeting
impl<'a> DisplayMultiView<()> for PowerBudgeting {}
impl<'a> fmt::Display for MultiView<&'a PowerBudgeting, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Power Budgeting <?>")
    }
}

// 0005h Root Complex Link Declaration
mod rclink;

// 0006h Root Complex Internal Link Control
impl<'a> fmt::Display for View<&'a RootComplexInternalLinkControl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Complex Internal Link <?>")
    }
}

// 0007h Root Complex Event Collector Endpoint Association
impl<'a> fmt::Display for View<&'a RootComplexEventCollectorEndpointAssociation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Complex Event Collector <?>")
    }
}

// 0008h Multi-Function Virtual Channel (MFVC)
impl<'a> fmt::Display for View<&'a MultifunctionVirtualChannel<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Multi-Function Virtual Channel <?>")
    }
}
impl<'a> fmt::Display for View<&'a MultifunctionVirtualChannelError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Multi-Function Virtual Channel <?>")
    }
}

// 000Ah Root Complex Register Block (RCRB) Header
impl<'a> fmt::Display for View<&'a RootComplexRegisterBlockHeader> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Complex Register Block <?>")
    }
}

// 000Bh Vendor-Specific Extended Capability (VSEC)
impl<'a> DisplayMultiView<()> for VendorSpecificExtendedCapability<'a> {}
impl<'a> fmt::Display for MultiView<&'a VendorSpecificExtendedCapability<'a>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = &self.data.header;
        writeln!(
            f,
            "Vendor Specific Information: ID={:04x} Rev={} Len={:03x} <?>",
            h.vsec_id, h.vsec_rev, h.vsec_length
        )
    }
}

// 000Ch Configuration Access Correlation (CAC)
impl<'a> fmt::Display for View<&'a ConfigurationAccessCorrelation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, " <?>")
    }
}

// 000Dh Access Control Services (ACS)
impl<'a> DisplayMultiView<Verbose> for AccessControlServices<'a> {}
impl<'a> fmt::Display for MultiView<&'a AccessControlServices<'a>, Verbose> {
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

// 000Eh Alternative Routing-ID Interpretation (ARI)
impl<'a> DisplayMultiView<Verbose> for AlternativeRoutingIdInterpretation {}
impl<'a> fmt::Display for MultiView<&'a AlternativeRoutingIdInterpretation, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Alternative Routing-ID Interpretation (ARI)")?;
        if verbose < 2 {
            return Ok(());
        }
        let AlternativeRoutingIdInterpretation {
            ari_capability: caps,
            ari_control: ctrl,
        } = self.data;
        writeln!(
            f,
            "\t\tARICap:\tMFVC{} ACS{}, Next Function: {}",
            caps.mfvc_function_groups_capability
                .display(BoolView::PlusMinus),
            caps.acs_function_groups_capability
                .display(BoolView::PlusMinus),
            caps.next_function_number,
        )?;
        writeln!(
            f,
            "\t\tARICtl:\tMFVC{} ACS{}, Function Group: {}",
            ctrl.mfvc_function_groups_enable
                .display(BoolView::PlusMinus),
            ctrl.acs_function_groups_enable.display(BoolView::PlusMinus),
            ctrl.function_group,
        )
    }
}

// 000Fh Address Translation Services (ATS)
impl<'a> DisplayMultiView<Verbose> for AddressTranslationServices {}
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
        writeln!(
            f,
            "\t\tATSCap:\tInvalidate Queue Depth: {:02x}",
            caps.invalidate_queue_depth,
        )?;
        writeln!(
            f,
            "\t\tATSCtl:\tEnable{}, Smallest Translation Unit: {:02x}",
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.smallest_translation_unit,
        )
    }
}

// 0010h Single Root I/O Virtualization (SR-IOV)
mod sr_iov;

// 0011h Multi-Root I/O Virtualization (MR-IOV) – defined in the Multi-Root I/O Virtualization and Sharing Specification
// 0012h Multicast

// 0013h Page Request Interface (PRI)
impl<'a> DisplayMultiView<Verbose> for PageRequestInterface {}
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
        writeln!(
            f,
            "\t\tPRICtl: Enable{} Reset{}",
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.reset.display(BoolView::PlusMinus),
        )?;
        writeln!(
            f,
            "\t\tPRISta: RF{} UPRGI{} Stopped{}",
            sta.response_failure.display(BoolView::PlusMinus),
            sta.unexpected_page_request_group_index
                .display(BoolView::PlusMinus),
            sta.stopped.display(BoolView::PlusMinus),
        )?;
        writeln!(
            f,
            "\t\tPage Request Capacity: {:08x}, Page Request Allocation: {:08x}",
            outstanding_page_request_capacity, outstanding_page_request_allocation,
        )
    }
}

// 0014h Reserved for AMD
// 0015h Resizable BAR
// 0016h Dynamic Power Allocation (DPA)

// 0017h TPH Requester
impl<'a> DisplayMultiView<Verbose> for TphRequester<'a> {}
impl<'a> fmt::Display for MultiView<&'a TphRequester<'a>, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Transaction Processing Hints")?;
        if verbose < 2 {
            return Ok(());
        }
        let TphRequester {
            tph_requester_capability: caps,
            ..
        } = self.data;
        if caps.interrupt_vector_mode_supported {
            writeln!(f, "\t\tInterrupt vector mode supported")?;
        }
        if caps.device_specific_mode_supported {
            writeln!(f, "\t\tDevice specific mode supported")?;
        }
        if caps.extended_tph_requester_supported {
            writeln!(f, "\t\tExtended requester support")?;
        }
        match caps.st_table {
            StTable::NotPresent => writeln!(f, "\t\tNo steering table available"),
            StTable::Valid { .. } => writeln!(f, "\t\tSteering table in TPH capability structure"),
            StTable::Invalid { .. } => {
                writeln!(f, "\t\tSteering table in TPH capability structure")
            }
            StTable::MsiXTable { .. } => writeln!(f, "\t\tSteering table in MSI-X table"),
            StTable::Reserved => writeln!(f, "\t\tReserved steering table location"),
        }
    }
}

// 0018h Latency Tolerance Reporting (LTR)
impl<'a> DisplayMultiView<Verbose> for LatencyToleranceReporting {}
impl<'a> fmt::Display for MultiView<&'a LatencyToleranceReporting, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Latency Tolerance Reporting")?;
        if verbose < 2 {
            return Ok(());
        }
        let LatencyToleranceReporting {
            max_snoop_latency: snoop,
            max_no_snoop_latency: nosnoop,
        } = self.data;

        // lspci has simple value to scale multiplication, although scale can't be > 5
        fn calc(value: u16, scale: u8) -> u64 {
            // PCI_LTR_VALUE_MASK
            let value = (value as u64) & 0x3ff;
            let scale = 1u32.wrapping_shl(5 * (scale as u32));
            value * (scale as u64)
        }

        writeln!(
            f,
            "\t\tMax snoop latency: {}ns",
            calc(snoop.value, snoop.scale)
        )?;
        writeln!(
            f,
            "\t\tMax no snoop latency: {}ns",
            calc(nosnoop.value, nosnoop.scale)
        )
    }
}

// 0019h Secondary PCI Express
impl<'a> DisplayMultiView<Verbose> for SecondaryPciExpress<'a> {}
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
        writeln!(
            f,
            "\t\tLnkCtl3: LnkEquIntrruptEn{} PerformEqu{}",
            ctrl.link_equalization_request_interrupt_enable
                .display(BoolView::PlusMinus),
            ctrl.perform_equalization.display(BoolView::PlusMinus),
        )?;
        let mut lane_err_sta = lane_error_status.0 as u16;
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

// 001Ah Protocol Multiplexing (PMUX)

// 001Bh Process Address Space ID (PASID)
impl<'a> DisplayMultiView<Verbose> for ProcessAddressSpaceId {}
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
        writeln!(
            f,
            "\t\tPASIDCap: Exec{} Priv{}, Max PASID Width: {:02x}",
            caps.execute_permission_supported
                .display(BoolView::PlusMinus),
            caps.privileged_mode_supported.display(BoolView::PlusMinus),
            caps.max_pasid_width,
        )?;
        writeln!(
            f,
            "\t\tPASIDCtl: Enable{} Exec{} Priv{}",
            ctrl.pasid_enable.display(BoolView::PlusMinus),
            ctrl.execute_permission_enable.display(BoolView::PlusMinus),
            ctrl.privileged_mode_enable.display(BoolView::PlusMinus),
        )?;
        Ok(())
    }
}

// 001Ch LN Requester (LNR)

struct DpcView {
    verbose: usize,
    maybe_raw_dpc_trigger_reason_extension: Option<u8>,
}
// 001Dh Downstream Port Containment (DPC)
impl<'a> DisplayMultiView<DpcView> for DownstreamPortContainment {}
impl<'a> fmt::Display for MultiView<&'a DownstreamPortContainment, DpcView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DpcView {
            verbose,
            maybe_raw_dpc_trigger_reason_extension,
        } = self.view;
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
            maybe_raw_dpc_trigger_reason_extension.unwrap_or(0),
            dpc_status.rp_pio_first_error_pointer,
        )?;
        writeln!(f, "\t\tSource:\t{:04x}", dpc_error_source_id)
    }
}

// 001Eh L1 PM Substates
impl<'a> DisplayMultiView<Verbose> for L1PmSubstates {}
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
        writeln!(
            f,
            "\t\tL1SubCap: PCI-PM_L1.2{} PCI-PM_L1.1{} ASPM_L1.2{} ASPM_L1.1{} L1_PM_Substates{}",
            caps.pci_pm_l1_2_supported.display(BoolView::PlusMinus),
            caps.pci_pm_l1_1_supported.display(BoolView::PlusMinus),
            caps.aspm_l1_2_supported.display(BoolView::PlusMinus),
            caps.aspm_l1_1_supported.display(BoolView::PlusMinus),
            // caps.l1_pm_substates_supported.display(BoolView::PlusMinus),
            // There is the bug in lspci: PCI_L1PM_SUBSTAT_CAP_L1PM_SUPP should be 0x10 not 0x16
            (caps.pci_pm_l1_1_supported
                || caps.aspm_l1_2_supported
                || caps.l1_pm_substates_supported)
                .display(BoolView::PlusMinus),
        )?;
        let is_l1_2_supported = caps.pci_pm_l1_2_supported || caps.aspm_l1_2_supported;
        if is_l1_2_supported {
            write!(
                f,
                "\t\t\t  PortCommonModeRestoreTime={}us ",
                caps.port_common_mode_restore_time
            )?;
            if let Some(time) = caps.port_t_power_on.value() {
                writeln!(f, "PortTPowerOnTime={}us", time)?;
            } else {
                writeln!(f, "PortTPowerOnTime=<error>")?;
            };
        }
        writeln!(
            f,
            "\t\tL1SubCtl1: PCI-PM_L1.2{} PCI-PM_L1.1{} ASPM_L1.2{} ASPM_L1.1{}",
            ctl1.pci_pm_l1_2_enable.display(BoolView::PlusMinus),
            ctl1.pci_pm_l1_1_enable.display(BoolView::PlusMinus),
            ctl1.aspm_l1_2_enable.display(BoolView::PlusMinus),
            ctl1.aspm_l1_1_enable.display(BoolView::PlusMinus),
        )?;
        if is_l1_2_supported {
            write!(
                f,
                "\t\t\t   T_CommonMode={}us",
                ctl1.common_mode_restore_time
            )?;
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
impl<'a> DisplayMultiView<Verbose> for PrecisionTimeMeasurement {}
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
        writeln!(
            f,
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
        writeln!(
            f,
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

// 0020h PCI Express over M-PHY (M-PCIe)
// 0021h FRS Queueing
// 0022h Readiness Time Reporting
// 0023h Designated Vendor-Specific Extended Capability
// 0024h VF Resizable BAR
// 0025h Data Link Feature
// 0026h Physical Layer 16.0 GT/s
// 0027h Lane Margining at the Receiver
// 0028h Hierarchy ID
// 0029h Native PCIe Enclosure Management (NPEM)
// 002Ah Physical Layer 32.0 GT/s
// 002Bh Alternate Protocol
// 002Ch System Firmware Intermediary (SFI)
