use core::fmt;

use pcics::capabilities::{
    message_signaled_interrups::MessageAddress,
    msi_x::Bir,
    sata::BarLocation,
    slot_identification::ExpansionSlot,
    vendor_specific::{VendorSpecific, VendorSpecificError, Virtio},
    AdvancedFeatures, Capability, CapabilityError, CapabilityKind, DebugPort,
    MessageSignaledInterrups, MsiX, Sata, SlotIdentification,
};

use super::{Flag, Simple, Verbose, View};
use crate::{
    access::Access,
    device::{Device, DeviceDependentRegion},
    misc::pnp::PlugAndPlayResource,
    names::VendorDeviceSubsystem,
    view::{DisplayMultiView, MultiView},
};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ViewArgs<'a> {
    pub verbose: usize,
    pub as_numbers: usize,
    pub device: &'a Device,
    pub vds: &'a VendorDeviceSubsystem,
    pub access: &'a Access,
}

impl<'a> fmt::Display for View<Capability<'a>, &'a ViewArgs<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Capability { pointer, ref kind } = self.data;
        let &ViewArgs {
            verbose,
            as_numbers,
            device,
            vds,
            access,
            ..
        } = self.args;
        let raw_data_offset = (pointer as usize).saturating_sub(DeviceDependentRegion::OFFSET);
        let raw_data = device
            .device_dependent_region
            .as_ref()
            .and_then(|s| s.get(raw_data_offset..))
            .unwrap_or_default();
        write!(f, "\tCapabilities: [{:02x}] ", pointer)?;
        match kind {
            CapabilityKind::NullCapability => writeln!(f, "Null"),
            CapabilityKind::PowerManagementInterface(pmi) => {
                let view = pmi::View {
                    pmi,
                    raw_data,
                    verbose,
                };
                write!(f, "{}", view)
            }
            CapabilityKind::AcceleratedGraphicsPort(data) => {
                write!(f, "{}", Verbose { data, verbose })
            }
            CapabilityKind::VitalProductData(data) => {
                let pnp = &access.vital_product_data(device.address.clone()).ok();
                let pnp = pnp.as_ref().map(|data| PlugAndPlayResource::new(data));
                let args = vpd::ViewArgs { verbose, pnp };
                write!(f, "{}", View { data, args })
            }
            CapabilityKind::SlotIdentification(si) => {
                write!(f, "{}", Simple(si))
            }
            CapabilityKind::MessageSignaledInterrups(data) => {
                write!(f, "{}", Verbose { data, verbose })
            }
            CapabilityKind::CompactPciHotSwap(_) => writeln!(f, "CompactPCI hot-swap <?>"),
            CapabilityKind::PciX(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::PciXBridge(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::Hypertransport(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::VendorSpecific(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::DebugPort(dp) => write!(f, "{}", Simple(dp)),
            CapabilityKind::CompactPciResourceControl(_) => {
                writeln!(f, "CompactPCI central resource control <?>")
            }
            CapabilityKind::PciHotPlug(_) => writeln!(f, "Hot-plug capable"),
            CapabilityKind::BridgeSubsystemVendorId(data) => {
                writeln!(
                    f,
                    "{}",
                    View {
                        data,
                        args: ssvid::ViewArgs { as_numbers, vds }
                    }
                )
            }
            CapabilityKind::Agp8x(_) => writeln!(f, "AGP3 <?>"),
            CapabilityKind::SecureDevice(_) => writeln!(f, "Secure device <?>"),
            CapabilityKind::PciExpress(c) => {
                let view = PciExpressView {
                    pointer,
                    verbose,
                    device,
                };
                write!(f, "{}", c.display(view))
            }
            CapabilityKind::MsiX(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::Sata(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::AdvancedFeatures(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::EnhancedAllocation(data) => write!(f, "{}", Verbose { data, verbose }),
            CapabilityKind::FlatteningPortalBridge(fpb) => {
                writeln!(f, "Capability ID 0x15 [{:04x}]", fpb.reserved)
            }
            CapabilityKind::Reserved(cid) => writeln!(f, "{:#02x}", cid),
        }
    }
}

impl fmt::Display for Simple<CapabilityError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            CapabilityError::VendorSpecific { source: data, ptr } => {
                write!(f, "\tCapabilities: [{:02x}] {}", ptr, Simple(data))
            }
            err => write!(f, "\tCapabilities: {}", err),
        }
    }
}

// 01h PCI Power Management Interface
mod pmi;

// 02h AGP
mod agp;

// 03h VPD
mod vpd;

// 04h Slot Identification
impl<'a> fmt::Display for Simple<&'a SlotIdentification> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SlotIdentification {
            expansion_slot:
                ExpansionSlot {
                    expansion_slots_provided,
                    first_in_chassis,
                },
            chassis_number,
        } = self.0;
        writeln!(
            f,
            "Slot ID: {} slots, First{}, chassis {:02x}",
            expansion_slots_provided,
            Flag(*first_in_chassis),
            chassis_number,
        )
    }
}

// 05h Message Signaled Interrupts
impl<'a> fmt::Display for Verbose<&'a MessageSignaledInterrups> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MessageSignaledInterrups {
            message_control: ref ctrl,
            message_address: ref addr,
            message_data: data,
            mask_bits: mask,
            pending_bits: pend,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(
            f,
            "MSI: Enable{} Count={}/{} Maskable{} 64bit{}",
            Flag(ctrl.msi_enable),
            ctrl.multiple_message_enable.number_of_vectors(),
            ctrl.multiple_message_capable.number_of_vectors(),
            Flag(ctrl.per_vector_masking_capable),
            Flag(matches!(addr, MessageAddress::Qword(_))),
        )?;
        if verbose < 2 {
            return Ok(());
        }
        match addr {
            MessageAddress::Dword(v) => {
                writeln!(f, "\t\tAddress: {:08x}  Data: {:04x}", v, data)?;
            }
            MessageAddress::Qword(v) => {
                writeln!(f, "\t\tAddress: {:016x}  Data: {:04x}", v, data)?;
            }
        }
        if let (Some(m), Some(p)) = (mask, pend) {
            writeln!(f, "\t\tMasking: {:08x}  Pending: {:08x}", m, p)?;
        }
        Ok(())
    }
}

// 07h PCI-X
mod pci_x;

// 08h HyperTransport
mod hypertransport;

// 09h Vendor Specific
impl<'a> fmt::Display for Verbose<&'a VendorSpecific<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &Verbose { data, verbose } = self;
        write!(f, "Vendor Specific Information: ")?;
        match data {
            VendorSpecific::Virtio(virtio) => {
                let (bar, offset, size, multiplier) = match virtio {
                    Virtio::CommonCfg { bar, offset, size } => {
                        writeln!(f, "VirtIO: CommonCfg",)?;
                        (bar, offset, size, &None)
                    }
                    Virtio::Notify {
                        bar,
                        offset,
                        size,
                        multiplier,
                    } => {
                        writeln!(f, "VirtIO: Notify",)?;
                        (bar, offset, size, multiplier)
                    }
                    Virtio::Isr { bar, offset, size } => {
                        writeln!(f, "VirtIO: ISR",)?;
                        (bar, offset, size, &None)
                    }
                    Virtio::DeviceCfg { bar, offset, size } => {
                        writeln!(f, "VirtIO: DeviceCfg",)?;
                        (bar, offset, size, &None)
                    }
                    Virtio::Unknown { bar, offset, size } => {
                        writeln!(f, "VirtIO: <unknown>",)?;
                        (bar, offset, size, &None)
                    }
                };
                if verbose < 2 {
                    return Ok(());
                }
                write!(f, "\t\tBAR={} offset={:08x} size={:08x}", bar, offset, size)?;
                if let Some(multiplier) = multiplier {
                    write!(f, " multiplier={:08x}", multiplier)?;
                }
                writeln!(f)
            }
            VendorSpecific::Unspecified(slice) => writeln!(f, "Len={:02x} <?>", slice.len() + 3),
        }
    }
}

impl<'a> fmt::Display for Simple<&'a VendorSpecificError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            VendorSpecificError::LengthUnreadable => {
                writeln!(f, "Vendor Specific Information: Len=00 <?>")
            }
            VendorSpecificError::Length { val } => {
                writeln!(f, "Vendor Specific Information: Len={:02x} <?>", val)
            }
            VendorSpecificError::Data { size } => {
                writeln!(f, "Vendor Specific Information: Len={:02x} <?>", size + 2)
            }
            VendorSpecificError::Virtio => {
                writeln!(f, "Vendor Specific Information: VirtIO: <unknown>")
            }
        }
    }
}

// 0Ah Debug port
impl<'a> fmt::Display for Simple<&'a DebugPort> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DebugPort { offset, bar_number } = self.0;
        writeln!(f, "Debug port: BAR={} offset={:04x}", bar_number, offset)
    }
}

// 0Dh PCI Bridge Subsystem Vendor ID
mod ssvid;

// 10h PCI Express
mod pci_express;
pub use pci_express::*;

// 11h MSI-X
impl<'a> fmt::Display for Verbose<&'a MsiX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose {
            data:
                MsiX {
                    message_control: ctrl,
                    table: tbl,
                    pending_bit_array: pba,
                },
            verbose,
        } = self;
        writeln!(
            f,
            "MSI-X: Enable{} Count={} Masked{}",
            Flag(ctrl.msi_x_enable),
            ctrl.table_size + 1,
            Flag(ctrl.function_mask),
        )?;
        if verbose > &1 {
            write!(
                f,
                "\t\tVector table: BAR={} offset={:08x}\n\t\tPBA: BAR={} offset={:08x}\n",
                tbl.bir.display(()),
                tbl.offset,
                pba.bir.display(()),
                pba.offset,
            )?;
        }
        Ok(())
    }
}

// 12h Serial ATA Data/Index Configuration
impl<'a> fmt::Display for Verbose<&'a Sata> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose {
            data:
                Sata {
                    revision,
                    bar_location,
                    bar_offset,
                },
            verbose,
        } = self;
        write!(f, "SATA HBA v{}.{}", revision.major, revision.minor)?;
        if verbose < &2 {
            writeln!(f)?;
            return Ok(());
        }
        match bar_location {
            BarLocation::Bar0 => writeln!(f, " BAR0 Offset={:08x}", bar_offset.0),
            BarLocation::Bar1 => writeln!(f, " BAR1 Offset={:08x}", bar_offset.0),
            BarLocation::Bar2 => writeln!(f, " BAR2 Offset={:08x}", bar_offset.0),
            BarLocation::Bar3 => writeln!(f, " BAR3 Offset={:08x}", bar_offset.0),
            BarLocation::Bar4 => writeln!(f, " BAR4 Offset={:08x}", bar_offset.0),
            BarLocation::Bar5 => writeln!(f, " BAR5 Offset={:08x}", bar_offset.0),
            BarLocation::SataCapability1 => writeln!(f, " InCfgSpace"),
            BarLocation::Reserved(v) => writeln!(f, " BAR??{}", v),
        }
    }
}

// 13h Advanced Features (AF)
impl<'a> fmt::Display for Verbose<&'a AdvancedFeatures> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose {
            data:
                AdvancedFeatures {
                    capabilities: caps,
                    control: ctrl,
                    status: st,
                    ..
                },
            verbose,
        } = self;
        writeln!(f, "PCI Advanced Features")?;
        if verbose > &1 {
            write!(
                f,
                "\t\tAFCap: TP{} FLR{}\n\t\tAFCtrl: FLR{}\n\t\tAFStatus: TP{}\n",
                Flag(caps.transactions_pending),
                Flag(caps.function_level_reset),
                Flag(ctrl.initiate_flr),
                Flag(st.transactions_pending),
            )?;
        }
        Ok(())
    }
}

// 14h Enhanced Allocation
mod ea;

impl DisplayMultiView<()> for Bir {}
impl fmt::Display for MultiView<&Bir, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = match self.data {
            Bir::Bar10h => 0,
            Bir::Bar14h => 1,
            Bir::Bar18h => 2,
            Bir::Bar1Ch => 3,
            Bir::Bar20h => 4,
            Bir::Bar24h => 5,
            Bir::Reserved(v) => *v,
        };
        write!(f, "{}", n)
    }
}

#[cfg(test)]
mod tests {
    use crate::device::Address;
    use crate::device::ConfigurationSpace;
    use crate::device::Device;
    use crate::device::DDR_OFFSET;
    use crate::device::ECS_OFFSET;
    use crate::names::Names;
    use pcics::capabilities::VendorSpecific;
    use pcics::Capabilities;
    use pcics::Header;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn vendor_specific() {
        // Capabilities: [b4] Vendor Specific Information: VirtIO: Notify
        //         BAR=4 offset=00003000 size=00001000 multiplier=00000004
        let mut header: Header = [0; 0x40].try_into().unwrap();
        header.vendor_id = 0x1af4;
        header.device_id = 0x1045;
        let data = [
            0x09, // Vendor Specific ID = 0x09
            0xa4, // next capabilities pointer
            0x14, // length = 20
            0x02, // Virtio type
            0x04, // BAR
            0x00, 0x00, 0x00, // skipped
            0x00, 0x30, 0x00, 0x00, // offset
            0x00, 0x10, 0x00, 0x00, // size
            0x04, 0x00, 0x00, 0x00, // multiplier
        ];
        let data = &VendorSpecific::try_new(&data[2..], &header).unwrap();
        assert_eq!(
            "Vendor Specific Information: VirtIO: Notify\n\
            \t\tBAR=4 offset=00003000 size=00001000 multiplier=00000004\n",
            format!("{}", Verbose { data, verbose: 2 })
        );
    }

    #[test]
    fn capabilities() {
        // Capabilities: [50] Power Management version 3
        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=55mA PME(D0-,D1-,D2-,D3hot+,D3cold+)
        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
        // Capabilities: [80] Vendor Specific Information: Len=14 <?>
        // Capabilities: [60] MSI: Enable+ Count=1/1 Maskable- 64bit+
        //         Address: 00000000fee00578  Data: 0000
        let data = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/config"
        ));
        let names = Names::init().unwrap();
        let vds = &names.vendor_device_subsystem();
        let device = {
            let cs: ConfigurationSpace = data.as_slice().try_into().unwrap();
            let address: Address = "00:1f.3".parse().unwrap();
            &Device::new(address, cs)
        };
        let ddr = &data[DDR_OFFSET..ECS_OFFSET];
        let mut header: Header = data.as_slice().try_into().unwrap();
        header.capabilities_pointer = 0x50;
        let caps = Capabilities::new(ddr, &header);
        let args = &ViewArgs {
            device,
            vds,
            verbose: 2,
            as_numbers: 0,
            access: &Default::default(),
        };
        let s = caps
            .map(|cap| match cap {
                Ok(data) => View { data, args }.to_string(),
                Err(e) => e.to_string(),
            })
            .collect::<String>();
        let result = s.lines().collect::<Vec<_>>();
        let sample = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:9dc8/caps.lspci.vvv.txt"
        ))
        .lines()
        .collect::<Vec<_>>();
        assert_eq!(sample, result);
    }
}
