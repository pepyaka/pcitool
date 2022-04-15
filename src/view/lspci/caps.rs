use core::fmt;

use pcics::capabilities::msi_x::Bir;
use pcics::capabilities::sata::BarLocation;
use pcics::capabilities::{
    Capability, CapabilityKind, PowerManagementInterface, VitalProductData,
    MessageSignaledInterrups, BridgeSubsystemVendorId, MsiX, AdvancedFeatures, Sata, DebugPort,
    SlotIdentification,
};

use crate::pciids::VendorDeviceSubsystem;
use crate::view::{
    BoolView,
    DisplayMultiViewBasic,
    MultiView,
    Verbose,
};
use crate::device::Device;
use super::BasicView;


pub struct CapabilityView<'a> {
    pub view: &'a BasicView,
    pub device: &'a Device,
    pub vds: &'a VendorDeviceSubsystem,
}

impl<'a> DisplayMultiViewBasic<&'a CapabilityView<'a>> for Capability<'a> {}
impl<'a> fmt::Display for MultiView<&'a Capability<'a>, &'a CapabilityView<'a>> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       let CapabilityView {
           view: BasicView {
               verbose,
               as_numbers,
               ..
           },
           device, vds,
       } = &self.view;
       let Capability { pointer, kind } = &self.data;
       let verbose = Verbose(*verbose);
       write!(f, "\tCapabilities: [{:02x}] ", self.data.pointer)?;
       match kind {
           CapabilityKind::NullCapability => writeln!(f, "Null"),
           CapabilityKind::PowerManagementInterface(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::VitalProductData(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::MessageSignaledInterrups(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::CompactPciHotSwap(_) => writeln!(f, "CompactPCI hot-swap <?>"),
           CapabilityKind::Hypertransport(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::VendorSpecific(c) => {
               let vc = c.vendor_capability(device.header.vendor_id, device.header.device_id);
               write!(f, "{}", vc.display(()))
           },
           CapabilityKind::SlotIdentification(c) => write!(f, "{}", c.display(())),
           CapabilityKind::DebugPort(c) => write!(f, "{}", c.display(())),
           CapabilityKind::CompactPciResourceControl(_) =>
               writeln!(f, "CompactPCI central resource control <?>"),
           CapabilityKind::PciHotPlug(_) => writeln!(f, "Hot-plug capable"),
           CapabilityKind::BridgeSubsystemVendorId(c) =>
               writeln!(f, "{}", c.display((*as_numbers, vds))),
           CapabilityKind::Agp8x(_) => writeln!(f, "AGP3 <?>"),
           CapabilityKind::SecureDevice(_) => writeln!(f, "Secure device <?>"),
           CapabilityKind::PciExpress(c) => {
               let view = PciExpressView {
                   pointer: *pointer,
                   verbose: verbose.0,
                   device
               };
               write!(f, "{}", c.display(view))
           },
           CapabilityKind::MsiX(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::Sata(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::AdvancedFeatures(c) => write!(f, "{}", c.display(verbose)),
           CapabilityKind::Reserved(cid) => writeln!(f, "{:#02x}", cid),
       }
   }
}


// 01h PCI Power Management Interface
impl<'a> DisplayMultiViewBasic<Verbose> for PowerManagementInterface {}
impl<'a> fmt::Display for MultiView<&'a PowerManagementInterface, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (caps, ctrl, br) = 
            (&self.data.capabilities, &self.data.control, &self.data.bridge);
        let Verbose(verbose) = self.view;
        writeln!(f, "Power Management version {}", caps.version)?;
        if verbose < 2 {
            return Ok(())
        }
        writeln!(f, "\t\tFlags: PMEClk{} DSI{} D1{} D2{} AuxCurrent={} PME(D0{},D1{},D2{},D3hot{},D3cold{})",
            caps.pme_clock.display(BoolView::PlusMinus), 
            caps.device_specific_initialization.display(BoolView::PlusMinus), 
            caps.d1_support.display(BoolView::PlusMinus), 
            caps.d2_support.display(BoolView::PlusMinus), 
            caps.aux_current, 
            caps.pme_support.d0.display(BoolView::PlusMinus),
            caps.pme_support.d1.display(BoolView::PlusMinus),
            caps.pme_support.d2.display(BoolView::PlusMinus),
            caps.pme_support.d3_hot.display(BoolView::PlusMinus),
            caps.pme_support.d3_cold.display(BoolView::PlusMinus),
        )?;
        writeln!(f, "\t\tStatus: D{} NoSoftRst{} PME-Enable{} DSel={} DScale={:?} PME{}",
            ctrl.power_state as usize,
            ctrl.no_soft_reset.display(BoolView::PlusMinus),
            ctrl.pme_enabled.display(BoolView::PlusMinus),
            u8::from(ctrl.data_select),
            ctrl.data_scale as usize,
            ctrl.pme_status.display(BoolView::PlusMinus),
        )?;
        if br.bpcc_enabled || br.b2_b3 || br.reserved != 0 {
            writeln!(f, "\t\tBridge: PM{} B3{}",
                br.bpcc_enabled.display(BoolView::PlusMinus),
                (!br.b2_b3).display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
}

// 03h VPD
impl<'a> DisplayMultiViewBasic<Verbose> for VitalProductData {}
impl<'a> fmt::Display for MultiView<&'a VitalProductData, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        writeln!(f, "Vital Product Data")?;
        if verbose < 2 {
            return Ok(())
        }
        // TODO: Iterate through all VPD addresses

        writeln!(f, "\t\tNot readable")?;
        Ok(())
    }
}

// 04h Slot Identification
impl<'a> DisplayMultiViewBasic<()> for SlotIdentification {}
impl<'a> fmt::Display for MultiView<&'a SlotIdentification, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Slot ID: {} slots, First{}, chassis {:02x}",
            self.data.expansion_slot.expansion_slots_provided,
            self.data.expansion_slot.first_in_chassis.display(BoolView::PlusMinus),
            self.data.chassis_number,
        )
    }
}

// 05h Message Signaled Interrupts
impl<'a> DisplayMultiViewBasic<Verbose> for MessageSignaledInterrups {}
impl<'a> fmt::Display for MultiView<&'a MessageSignaledInterrups, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (ctrl, addr, data, mask, pend) = (
            &self.data.message_control, 
            &self.data.message_address, 
            &self.data.message_data, 
            &self.data.mask_bits, 
            &self.data.pending_bits
        );
        let Verbose(verbose) = self.view;
        writeln!(f, "MSI: Enable{} Count={}/{} Maskable{} 64bit{}", 
            ctrl.enable.display(BoolView::PlusMinus),
            ctrl.multiple_message_enable as u8,
            ctrl.multiple_message_capable as u8,
            ctrl.per_vector_masking_capable.display(BoolView::PlusMinus),
            matches!(addr, MessageAddress::Qword(_)).display(BoolView::PlusMinus),
        )?;
        if verbose < 2 {
            return Ok(())
        }
        match addr {
            MessageAddress::Dword(v) => {
                writeln!(f, "\t\tAddress: {:08x}  Data: {:04x}", v, data)?;
            },
            MessageAddress::Qword(v) => {
                writeln!(f, "\t\tAddress: {:016x}  Data: {:04x}", v, data)?;
            },
        }
        if let (Some(m), Some(p)) = (mask, pend) {
            writeln!(f, "\t\tMasking: {:08x}  Pending: {:08x}", m, p)?;
        }
        Ok(())
    }
}


// 08h HyperTransport
mod hypertransport;

// 09h Vendor Specific
// Used data from internal VendorCapabilty struct
impl<'a> DisplayMultiViewBasic<()> for VendorCapabilty<'a> {}
impl<'a> fmt::Display for MultiView<&'a VendorCapabilty<'a>, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vendor Specific Information: ")?;
        match self.data {
            VendorCapabilty::Virtio(virtio) => match virtio {
                Virtio::CommonCfg { bar, offset, size } => write!(f, 
                    "VirtIO: CommonCfg\n\t\tBAR={} offset={:08x} size={:08x}\n", 
                    bar, offset, size
                ),
                Virtio::Notify { bar, offset, size, multiplier } => {
                    write!(f, 
                        "VirtIO: Notify\n\t\tBAR={} offset={:08x} size={:08x}", 
                        bar, offset, size, 
                    )?;
                    if let Some(multiplier) = multiplier {
                        write!(f, " multiplier={:08x}", multiplier)?;
                    }
                    writeln!(f)
                },
                Virtio::Isr { bar, offset, size } => write!(f, 
                    "VirtIO: ISR\n\t\tBAR={} offset={:08x} size={:08x}\n", 
                    bar, offset, size
                ),
                Virtio::DeviceCfg { bar, offset, size } => write!(f, 
                    "VirtIO: DeviceCfg\n\t\tBAR={} offset={:08x} size={:08x}\n", 
                    bar, offset, size
                ),
                Virtio::Unknown { bar, offset, size } => write!(f, 
                    "VirtIO: Unknown\n\t\tBAR={} offset={:08x} size={:08x}\n", 
                    bar, offset, size
                ),
            },
            VendorCapabilty::Unspecified(slice) =>
                writeln!(f, "Len={:02x} <?>", slice.len() + 1),
        }
    }
}

// 0Ah Debug port
impl<'a> DisplayMultiViewBasic<()> for DebugPort {}
impl<'a> fmt::Display for MultiView<&'a DebugPort, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Debug port: BAR={} offset={:04x}", self.data.bar_number, self.data.offset)
    }
}

// 0Dh PCI Bridge Subsystem Vendor ID
impl<'a> DisplayMultiViewBasic<(usize, &'a VendorDeviceSubsystem)> for BridgeSubsystemVendorId {}
impl<'a> fmt::Display for MultiView<&'a BridgeSubsystemVendorId, (usize, &'a VendorDeviceSubsystem)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (as_numbers, vds) = self.view;
        let BridgeSubsystemVendorId {
            subsystem_vendor_id: vendor_id,
            subsystem_id: device_id,
            ..
        } = self.data;
        let pciids_vendor = vds.lookup(*vendor_id, None, None);
        let pciids_device = vds.lookup(*vendor_id, *device_id, None);
        write!(f, "Subsystem:")?;
        match (as_numbers, pciids_vendor, pciids_device) {
            (0, Some(v), Some(d)) => write!(f, " {} {}", v, d),
            (0, Some(v), None) => write!(f, " {} Device {:04x}", v, device_id),
            (1, _, _) => write!(f, " {:04x}:{:04x}", vendor_id, device_id),
            (_, Some(v), Some(d)) => write!(f, " {} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id),
            (_, Some(v), None) => write!(f, " {} Device [{:04x}:{:04x}]", v, vendor_id, device_id),
            _ => write!(f, " [{:04x}:{:04x}]", vendor_id, device_id),
        }
    }
}


// 10h PCI Express 
mod pci_express;
pub use pci_express::*;
use pcics::capabilities::message_signaled_interrups::MessageAddress;
use pcics::capabilities::vendor_specific::{VendorCapabilty, Virtio};

// 11h MSI-X
impl DisplayMultiViewBasic<Verbose> for MsiX {}
impl fmt::Display for MultiView<&MsiX, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MsiX { message_control: ctrl, table: tbl, pending_bit_array: pba } = self.data;
        writeln!(f,
            "MSI-X: Enable{} Count={} Masked{}",
            ctrl.msi_x_enable.display(BoolView::PlusMinus),
            ctrl.table_size + 1,
            ctrl.function_mask.display(BoolView::PlusMinus),
        )?;
        if self.view.0 > 1 {
            write!(f,
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
impl DisplayMultiViewBasic<Verbose> for Sata {}
impl fmt::Display for MultiView<&Sata, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        let Sata {
            revision,
            bar_location,
            bar_offset,
        } = self.data;
        write!(f, "SATA HBA v{}.{}", revision.major, revision.minor)?;
        if verbose < 2 {
            writeln!(f)?;
            return Ok(())
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
impl DisplayMultiViewBasic<Verbose> for AdvancedFeatures {}
impl fmt::Display for MultiView<&AdvancedFeatures, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AdvancedFeatures { capabilities: caps, control: ctrl, status: st, .. } = self.data;
        writeln!(f, "PCI Advanced Features")?;
        if self.view.0 > 1 {
            write!(f,
                "\t\tAFCap: TP{} FLR{}\n\t\tAFCtrl: FLR{}\n\t\tAFStatus: TP{}\n",
                caps.transactions_pending.display(BoolView::PlusMinus),
                caps.function_level_reset.display(BoolView::PlusMinus),
                ctrl.initiate_flr.display(BoolView::PlusMinus),
                st.transactions_pending.display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
}


impl DisplayMultiViewBasic<()> for Bir {}
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
    use crate::device::ECS_OFFSET;
    use crate::device::DDR_OFFSET;
    use crate::pciids::PciIds;
    use pcics::Capabilities;
    use pcics::capabilities::VendorSpecific;
    use pretty_assertions::assert_eq;

    use super::*;
    use byte::{
        ctx::*,
        self,
        BytesExt,
    };

    #[test]
    fn power_management_interface() {
        let data = [0x02,0x7e,0x00,0x00,0x40,0x00];
        let pmi = data.read_with::<PowerManagementInterface>(&mut 0, LE).unwrap();

        let v1_result = pmi.display(Verbose(1)).to_string();
        let v1_sample = "\
            Power Management version 2\n\
        ";
        assert_eq!(v1_sample, v1_result, "-v");

        let v2_result = pmi.display(Verbose(2)).to_string();
        let v2_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3-\n\
        ";
        assert_eq!(v2_sample, v2_result, "-vv");

        let v3_result = pmi.display(Verbose(3)).to_string();
        let v3_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3-\n\
        ";
        assert_eq!(v3_sample, v3_result, "-vvv");
    }

    #[test]
    fn vendor_specific() {
        // Capabilities: [b4] Vendor Specific Information: VirtIO: Notify
        //         BAR=4 offset=00003000 size=00001000 multiplier=00000004
        let data = [
            0x09, // Vendor Specific ID = 0x09
            0xa4, // next capabilities pointer
            0x14, // length = 20
            0x02, // Virtio type
            0x04, // BAR
            0x00, 0x00, 0x00,       // skipped
            0x00, 0x30, 0x00, 0x00, // offset
            0x00, 0x10, 0x00, 0x00, // size
            0x04, 0x00, 0x00, 0x00  // multiplier
        ];
        let vs = VendorSpecific(&data[3..]);
        let vc = vs.vendor_capability(0x1af4, 0x1045);
        assert_eq!(
            "Vendor Specific Information: VirtIO: Notify\n\
            \t\tBAR=4 offset=00003000 size=00001000 multiplier=00000004\n",
            vc.display(()).to_string()
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
        let data =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/config"
            ));
        let vds = 
            &PciIds::new(include_str!("/usr/share/hwdata/pci.ids").lines())
                .collect::<(VendorDeviceSubsystem, _)>().0;
        let device = {
            let cs: ConfigurationSpace = data.read_with(&mut 0, LE).unwrap();
            let address: Address = "00:1f.3".parse().unwrap();
            &Device::new(address, cs)
        };
        let ddr = &data[DDR_OFFSET..ECS_OFFSET];
        let offset = data[0x34];
        let caps = Capabilities::new(ddr, offset);
        let view = CapabilityView {
            view: &BasicView { verbose: 3, ..Default::default() },
            device, vds,
        };
        let s = caps.map(|cap| {
                match cap {
                    Ok(val) => val.display(&view).to_string(),
                    Err(e) => e.to_string(),
                }
            })
            .collect::<String>();
        let result = s.lines()
            .collect::<Vec<_>>();
        let sample =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                "/tests/data/device/8086:9dc8/caps.lspci.vvv.txt"
            )).lines().collect::<Vec<_>>();
        assert_eq!(sample, result);
    }
}
