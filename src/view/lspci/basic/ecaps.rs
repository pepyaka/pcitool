use core::fmt;

use pcics::{
    capabilities::PciExpress,
    extended_capabilities::{
        multifunction_virtual_channel::MultifunctionVirtualChannelError, tph_requester::StTable,
        AccessControlServices, AddressTranslationServices, AlternativeRoutingIdInterpretation,
        ConfigurationAccessCorrelation, DataLinkFeature, DeviceSerialNumber,
        DownstreamPortContainment, DynamicPowerAllocation, ExtendedCapability,
        ExtendedCapabilityError, ExtendedCapabilityKind, FrsQueuing, HierarchyId, L1PmSubstates,
        LaneMarginingAtTheReceiver, LatencyToleranceReporting, LnRequester,
        MultiRootIoVirtualization, MultifunctionVirtualChannel, NativePcieEnclosureManagement,
        PageRequestInterface, PciExpressOverMphy, PhysicalLayer16GTps, PowerBudgeting,
        PrecisionTimeMeasurement, ProcessAddressSpaceId, ProtocolMultiplexing,
        ReadinessTimeReporting, RootComplexEventCollectorEndpointAssociation,
        RootComplexInternalLinkControl, RootComplexRegisterBlockHeader, SecondaryPciExpress,
        TphRequester, VendorSpecificExtendedCapability,
    },
};

use crate::{
    device::Device,
    view::{DisplayMultiView, MultiView},
};

use self::vc::VcView;

use super::{Flag, Simple, Verbose, View};

pub(super) struct ViewArgs<'a> {
    pub verbose: usize,
    pub device: &'a Device,
    pub maybe_pci_express: Option<&'a PciExpress>,
}

impl<'a> fmt::Display for View<ExtendedCapability<'a>, &'a ViewArgs<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &ViewArgs {
            verbose,
            device,
            maybe_pci_express,
        } = self.args;
        let ExtendedCapability {
            ref kind,
            version,
            offset,
        } = self.data;
        write!(f, "\tCapabilities: [{:03x}", offset)?;
        if verbose > 1 {
            write!(f, " v{}", version)?;
        }
        write!(f, "] ")?;

        match kind {
            // 0000h
            ExtendedCapabilityKind::Null => writeln!(f, "Null"),
            // 0001h
            ExtendedCapabilityKind::AdvancedErrorReporting(data) => {
                let is_type_root = maybe_pci_express
                    .filter(|pcie| pcie.device_type.is_root())
                    .is_some();
                let args = &aer::ViewArgs {
                    verbose,
                    is_type_root,
                };
                write!(f, "{}", View { data, args })
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
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0006h
            ExtendedCapabilityKind::RootComplexInternalLinkControl(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0007h
            ExtendedCapabilityKind::RootComplexEventCollectorEndpointAssociation(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0008h
            ExtendedCapabilityKind::MultifunctionVirtualChannel(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0009h
            ExtendedCapabilityKind::VirtualChannelMfvcPresent(c) => {
                write!(f, "{}", c.display(VcView { verbose, offset }))
            }
            // 000Ah
            ExtendedCapabilityKind::RootComplexRegisterBlockHeader(c) => {
                write!(f, "{}", Simple(c))
            }
            // 000Bh
            ExtendedCapabilityKind::VendorSpecificExtendedCapability(c) => {
                write!(f, "{}", c.display(()))
            }
            // 000Dh
            ExtendedCapabilityKind::AccessControlServices(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 000Eh
            ExtendedCapabilityKind::AlternativeRoutingIdInterpretation(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 000Fh
            ExtendedCapabilityKind::AddressTranslationServices(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0010h
            ExtendedCapabilityKind::SingleRootIoVirtualization(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0011h
            ExtendedCapabilityKind::MultiRootIoVirtualization(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0012h
            ExtendedCapabilityKind::Multicast(c) => {
                write!(
                    f,
                    "{}",
                    MulticastView {
                        data: c,
                        verbose,
                        maybe_device_type: maybe_pci_express.map(|pcie| &pcie.device_type)
                    }
                )
            }
            // 0013h
            ExtendedCapabilityKind::PageRequestInterface(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0015h
            ExtendedCapabilityKind::ResizableBar(data) => {
                write!(
                    f,
                    "{}",
                    rebar::View {
                        result: Ok(data),
                        verbose,
                        is_virtual: false,
                    }
                )
            }
            // 016h
            ExtendedCapabilityKind::DynamicPowerAllocation(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0017h
            ExtendedCapabilityKind::TphRequester(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0018h
            ExtendedCapabilityKind::LatencyToleranceReporting(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 0019h
            ExtendedCapabilityKind::SecondaryPciExpress(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 01Ah
            ExtendedCapabilityKind::ProtocolMultiplexing(c) => {
                write!(f, "{}", Simple(c))
            }
            // 001Bh
            ExtendedCapabilityKind::ProcessAddressSpaceId(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 01Ch
            ExtendedCapabilityKind::LnRequester(c) => {
                write!(f, "{}", Simple(c))
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
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 001Fh
            ExtendedCapabilityKind::PrecisionTimeMeasurement(c) => {
                write!(f, "{}", Verbose { data: c, verbose })
            }
            // 020h
            ExtendedCapabilityKind::PciExpressOverMphy(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0021h
            ExtendedCapabilityKind::FrsQueuing(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0022h
            ExtendedCapabilityKind::ReadinessTimeReporting(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0023h
            ExtendedCapabilityKind::DesignatedVendorSpecificExtendedCapability(data) => {
                write!(f, "{}", Verbose { data, verbose })
            }
            // 0024h
            ExtendedCapabilityKind::VfResizableBar(data) => {
                write!(
                    f,
                    "{}",
                    rebar::View {
                        result: Ok(data),
                        verbose,
                        is_virtual: true,
                    }
                )
            }
            // 0025h
            ExtendedCapabilityKind::DataLinkFeature(c) => write!(f, "{}", Simple(c)),
            // 0026h
            ExtendedCapabilityKind::PhysicalLayer16GTps(c) => write!(f, "{}", Simple(c)),
            // 0027h
            ExtendedCapabilityKind::LaneMarginingAtTheReceiver(c) => {
                write!(f, "{}", Simple(c))
            }
            // 0028h
            ExtendedCapabilityKind::HierarchyId(c) => write!(f, "{}", Simple(c)),
            // 0029h
            ExtendedCapabilityKind::NativePcieEnclosureManagement(c) => {
                write!(f, "{}", Simple(c))
            }

            _ => writeln!(f, "Extended Capability ID {:#x}", &self.data.id()),
        }
    }
}

impl fmt::Display for Verbose<ExtendedCapabilityError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
        let (offset, msg) = match &self.data {
            offset @ ExtendedCapabilityError::Offset => (&0, format!("{}", offset)),
            header @ ExtendedCapabilityError::Header { offset } => (offset, format!("{}", header)),
            header @ ExtendedCapabilityError::EmptyHeader { offset } => {
                (offset, format!("{}", header))
            }
            ExtendedCapabilityError::Data { offset, source } => (offset, format!("{}", source)),
            ExtendedCapabilityError::AdvancedErrorReporting { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::RootComplexLinkDeclaration {
                offset,
                source: ref data,
            } => (offset, format!("{}", Verbose { data, verbose })),
            ExtendedCapabilityError::SingleRootIoVirtualization { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::DownstreamPortContainment { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::MultifunctionVirtualChannel { offset, source } => {
                (offset, format!("{}", Simple(source)))
            }
            ExtendedCapabilityError::ResizableBar { offset, source } => (
                offset,
                format!(
                    "{}",
                    rebar::View {
                        result: Err(source),
                        verbose,
                        is_virtual: false,
                    }
                ),
            ),
            ExtendedCapabilityError::DynamicPowerAllocation { offset, source } => {
                (offset, format!("{}", source))
            }
            ExtendedCapabilityError::ProtocolMultiplexing { offset, source } => {
                (offset, format!("{}", source))
            }
            // 0023h
            ExtendedCapabilityError::DesignatedVendorSpecificExtendedCapability {
                offset,
                source: data,
            } => (offset, format!("{}", Simple(data))),
            // 0024h
            ExtendedCapabilityError::VfResizableBar { offset, source } => (
                offset,
                format!(
                    "{}",
                    rebar::View {
                        result: Err(source),
                        verbose,
                        is_virtual: true,
                    }
                ),
            ),
        };
        let ver = if verbose > 1 { " v0" } else { "" };
        write!(f, "\tCapabilities: [{:03x}{}] {}", offset, ver, msg)
    }
}

// 0001h Advanced Error Reporting (AER)
mod aer;

// 0002h Virtual Channel (VC)
// 0009h Virtual Channel (VC) – used if an MFVC Extended Cap structure is present in the device
mod vc;

// 0003h Device Serial Number
impl DisplayMultiView<()> for DeviceSerialNumber {}
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
impl DisplayMultiView<()> for PowerBudgeting {}
impl<'a> fmt::Display for MultiView<&'a PowerBudgeting, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Power Budgeting <?>")
    }
}

// 0005h Root Complex Link Declaration
mod rclink;

// 0006h Root Complex Internal Link Control
impl<'a> fmt::Display for Simple<&'a RootComplexInternalLinkControl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Complex Internal Link <?>")
    }
}

// 0007h Root Complex Event Collector Endpoint Association
impl<'a> fmt::Display for Simple<&'a RootComplexEventCollectorEndpointAssociation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Complex Event Collector <?>")
    }
}

// 0008h Multi-Function Virtual Channel (MFVC)
impl<'a> fmt::Display for Simple<&'a MultifunctionVirtualChannel<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Multi-Function Virtual Channel <?>")
    }
}
impl<'a> fmt::Display for Simple<&'a MultifunctionVirtualChannelError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Multi-Function Virtual Channel <?>")
    }
}

// 000Ah Root Complex Register Block (RCRB) Header
impl<'a> fmt::Display for Simple<&'a RootComplexRegisterBlockHeader> {
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
impl<'a> fmt::Display for Simple<&'a ConfigurationAccessCorrelation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, " <?>")
    }
}

// 000Dh Access Control Services (ACS)
impl<'a> fmt::Display for Verbose<&'a AccessControlServices<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AccessControlServices {
            acs_capability,
            acs_control,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Access Control Services")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(f,
            "\t\tACSCap:\tSrcValid{} TransBlk{} ReqRedir{} CmpltRedir{} UpstreamFwd{} EgressCtrl{} DirectTrans{}", 
            Flag(acs_capability.acs_source_validation),
            Flag(acs_capability.acs_translation_blocking),
            Flag(acs_capability.acs_p2p_request_redirect),
            Flag(acs_capability.acs_p2p_completion_redirect),
            Flag(acs_capability.acs_upstream_forwarding),
            Flag(acs_capability.acs_p2p_egress_control),
            Flag(acs_capability.acs_direct_translated_p2p),
        )?;
        writeln!(f,
            "\t\tACSCtl:\tSrcValid{} TransBlk{} ReqRedir{} CmpltRedir{} UpstreamFwd{} EgressCtrl{} DirectTrans{}", 
            Flag(acs_control.acs_source_validation_enable),
            Flag(acs_control.acs_translation_blocking_enable),
            Flag(acs_control.acs_p2p_request_redirect_enable),
            Flag(acs_control.acs_p2p_completion_redirect_enable),
            Flag(acs_control.acs_upstream_forwarding_enable),
            Flag(acs_control.acs_p2p_egress_control_enable),
            Flag(acs_control.acs_direct_translated_p2p_enable),
        )
    }
}

// 000Eh Alternative Routing-ID Interpretation (ARI)
impl<'a> fmt::Display for Verbose<&'a AlternativeRoutingIdInterpretation> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AlternativeRoutingIdInterpretation {
            ari_capability: caps,
            ari_control: ctrl,
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Alternative Routing-ID Interpretation (ARI)")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tARICap:\tMFVC{} ACS{}, Next Function: {}",
            Flag(caps.mfvc_function_groups_capability),
            Flag(caps.acs_function_groups_capability),
            caps.next_function_number,
        )?;
        writeln!(
            f,
            "\t\tARICtl:\tMFVC{} ACS{}, Function Group: {}",
            Flag(ctrl.mfvc_function_groups_enable),
            Flag(ctrl.acs_function_groups_enable),
            ctrl.function_group,
        )
    }
}

// 000Fh Address Translation Services (ATS)
impl<'a> fmt::Display for Verbose<&'a AddressTranslationServices> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AddressTranslationServices {
            ats_capability: caps,
            ats_control: ctrl,
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Address Translation Service (ATS)")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tATSCap:\tInvalidate Queue Depth: {:02x}",
            caps.invalidate_queue_depth,
        )?;
        writeln!(
            f,
            "\t\tATSCtl:\tEnable{}, Smallest Translation Unit: {:02x}",
            Flag(ctrl.enable),
            ctrl.smallest_translation_unit,
        )
    }
}

// 0010h Single Root I/O Virtualization (SR-IOV)
mod sr_iov;

// 0011h Multi-Root I/O Virtualization (MR-IOV)
impl<'a> fmt::Display for Simple<&'a MultiRootIoVirtualization> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Multi-Root I/O Virtualization <?>")
    }
}

// 0012h Multicast
mod multicast;
use multicast::MulticastView;

// 0013h Page Request Interface (PRI)
impl<'a> fmt::Display for Verbose<&'a PageRequestInterface> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PageRequestInterface {
            page_request_control: ctrl,
            page_request_status: sta,
            outstanding_page_request_capacity,
            outstanding_page_request_allocation,
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Page Request Interface (PRI)")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tPRICtl: Enable{} Reset{}",
            Flag(ctrl.enable),
            Flag(ctrl.reset),
        )?;
        writeln!(
            f,
            "\t\tPRISta: RF{} UPRGI{} Stopped{}",
            Flag(sta.response_failure),
            Flag(sta.unexpected_page_request_group_index),
            Flag(sta.stopped),
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
// 0024h VF Resizable BAR
mod rebar;

// 0016h Dynamic Power Allocation (DPA)
impl<'a> fmt::Display for Simple<&'a DynamicPowerAllocation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dynamic Power Allocation <?>")
    }
}

// 0017h TPH Requester
impl<'a> fmt::Display for Verbose<&'a TphRequester<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
        let TphRequester {
            tph_requester_capability: caps,
            ..
        } = self.data;
        writeln!(f, "Transaction Processing Hints")?;
        if verbose < 2 {
            return Ok(());
        }
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
impl<'a> fmt::Display for Verbose<&'a LatencyToleranceReporting> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
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
impl<'a> fmt::Display for Verbose<&'a SecondaryPciExpress<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SecondaryPciExpress {
            link_control_3: ctrl,
            lane_error_status,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Secondary PCI Express")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tLnkCtl3: LnkEquIntrruptEn{} PerformEqu{}",
            Flag(ctrl.link_equalization_request_interrupt_enable),
            Flag(ctrl.perform_equalization),
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
impl<'a> fmt::Display for Simple<&'a ProtocolMultiplexing<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Protocol Multiplexing <?>")
    }
}

// 001Bh Process Address Space ID (PASID)
impl<'a> fmt::Display for Verbose<&'a ProcessAddressSpaceId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
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
            Flag(caps.execute_permission_supported),
            Flag(caps.privileged_mode_supported),
            caps.max_pasid_width,
        )?;
        writeln!(
            f,
            "\t\tPASIDCtl: Enable{} Exec{} Priv{}",
            Flag(ctrl.pasid_enable),
            Flag(ctrl.execute_permission_enable),
            Flag(ctrl.privileged_mode_enable),
        )?;
        Ok(())
    }
}

// 001Ch LN Requester (LNR)
impl<'a> fmt::Display for Simple<&'a LnRequester> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "LN Requester <?>")
    }
}

struct DpcView {
    verbose: usize,
    maybe_raw_dpc_trigger_reason_extension: Option<u8>,
}
// 001Dh Downstream Port Containment (DPC)
impl DisplayMultiView<DpcView> for DownstreamPortContainment {}
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
            Flag(dpc_capability.rp_extensions_for_dpc),
            Flag(dpc_capability.poisoned_tlp_egress_blocking_supported),
            Flag(dpc_capability.dpc_software_triggering_supported),
            dpc_capability.rp_pio_log_size,
            Flag(dpc_capability.dl_active_err_cor_signaling_supported),
        )?;
        writeln!(f,
            "\t\tDpcCtl:\tTrigger:{:x} Cmpl{} INT{} ErrCor{} PoisonedTLP{} SwTrigger{} DL_ActiveErr{}",
            dpc_control.dpc_trigger_enable as usize,
            Flag(dpc_control.dpc_completion_control),
            Flag(dpc_control.dpc_interrupt_enable),
            Flag(dpc_control.dpc_err_cor_enable),
            Flag(dpc_control.poisoned_tlp_egress_blocking_enable),
            Flag(dpc_control.dpc_software_trigger),
            Flag(dpc_control.dl_active_err_cor_enable),
        )?;
        writeln!(f,
            "\t\tDpcSta:\tTrigger{} Reason:{:02x} INT{} RPBusy{} TriggerExt:{:02x} RP PIO ErrPtr:{:02x}",
            Flag(dpc_status.dpc_trigger_status),
            dpc_status.dpc_trigger_reason.value(),
            Flag(dpc_status.dpc_interrupt_status),
            Flag(dpc_status.dpc_rp_busy),
            maybe_raw_dpc_trigger_reason_extension.unwrap_or(0),
            dpc_status.rp_pio_first_error_pointer,
        )?;
        writeln!(f, "\t\tSource:\t{:04x}", dpc_error_source_id)
    }
}

// 001Eh L1 PM Substates
impl<'a> fmt::Display for Verbose<&'a L1PmSubstates> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
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
            Flag(caps.pci_pm_l1_2_supported),
            Flag(caps.pci_pm_l1_1_supported),
            Flag(caps.aspm_l1_2_supported),
            Flag(caps.aspm_l1_1_supported),
            // Flag(caps.l1_pm_substates_supported),
            // There is the bug in lspci: PCI_L1PM_SUBSTAT_CAP_L1PM_SUPP should be 0x10 not 0x16
            Flag(
                caps.pci_pm_l1_1_supported
                    || caps.aspm_l1_2_supported
                    || caps.l1_pm_substates_supported
            ),
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
            Flag(ctl1.pci_pm_l1_2_enable),
            Flag(ctl1.pci_pm_l1_1_enable),
            Flag(ctl1.aspm_l1_2_enable),
            Flag(ctl1.aspm_l1_1_enable),
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
impl<'a> fmt::Display for Verbose<&'a PrecisionTimeMeasurement> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
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
            Flag(caps.ptm_requester_capable),
            Flag(caps.ptm_responder_capable),
            Flag(caps.ptm_root_capable),
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
            Flag(ctrl.ptm_enable),
            Flag(ctrl.root_select),
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
impl<'a> fmt::Display for Simple<&'a PciExpressOverMphy> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "PCI Express over M_PHY <?>")
    }
}

// 0021h FRS Queueing
impl<'a> fmt::Display for Simple<&'a FrsQueuing> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "FRS Queueing <?>")
    }
}

// 0022h Readiness Time Reporting
impl<'a> fmt::Display for Simple<&'a ReadinessTimeReporting> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Readiness Time Reporting <?>")
    }
}

// 0023h Designated Vendor-Specific Extended Capability
mod dvsec;

// 0025h Data Link Feature
impl<'a> fmt::Display for Simple<&'a DataLinkFeature> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Data Link Feature <?>")
    }
}

// 0026h Physical Layer 16.0 GT/s
impl<'a> fmt::Display for Simple<&'a PhysicalLayer16GTps> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Physical Layer 16.0 GT/s <?>")
    }
}

// 0027h Lane Margining at the Receiver
impl<'a> fmt::Display for Simple<&'a LaneMarginingAtTheReceiver> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Lane Margining at the Receiver <?>")
    }
}

// 0028h Hierarchy ID
impl<'a> fmt::Display for Simple<&'a HierarchyId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hierarchy ID <?>")
    }
}

// 0029h Native PCIe Enclosure Management (NPEM)
impl<'a> fmt::Display for Simple<&'a NativePcieEnclosureManagement> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Native PCIe Enclosure Management <?>")
    }
}

// 002Ah Physical Layer 32.0 GT/s
// 002Bh Alternate Protocol
// 002Ch System Firmware Intermediary (SFI)
// 002Dh Shadow Functions
// 002Eh Data Object Exchange
// 002Fh Device 3
// 0030h Integrity and Data Encryption (IDE)
// 0031h Physical Layer 64.0 GT/s Capability
// 0032h Flit Logging
// 0033h Flit Performance Measurement
// 0034h Flit Error Injection
