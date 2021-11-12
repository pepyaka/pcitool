
use crate::view::*;
use crate::device::capabilities::*;
use super::BasicView;


impl<'a> DisplayMultiView<'a> for Capability<'a> {
    type Data = &'a Capability<'a>;
    type View = &'a BasicView;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}
impl<'a> fmt::Display for MultiView<&'a Capability<'a>, &'a BasicView> {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "\tCapabilities: [{:02x}] ", self.data.pointer)?;
       let verbose = Verbose(self.view.verbose);
       match &self.data.kind {
           CapabilityKind::PowerManagementInterface(c) =>
               write!(f, "{}", c.display(verbose)),
           CapabilityKind::VendorSpecific(c) => {
               let view = VendorSpecificView {
                   verbose: self.view.verbose,
                   vendor_id: 0,
                   device_id: 0,
               };
               write!(f, "{}", c.display(&view))
           },
           CapabilityKind::MessageSignaledInterrups(c) =>
               write!(f, "{}", c.display(verbose)),
           CapabilityKind::Reserved(cid) =>
               write!(f, "{:#02x}\n", cid),
       }
   }
}



impl<'a> fmt::Display for MultiView<&'a PowerManagementInterface, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (caps, ctrl, br, d) = 
            (&self.data.capabilities, &self.data.control, &self.data.bridge, &self.data.data);
        let Verbose(verbose) = self.view;
        write!(f, "Power Management version {}\n", caps.version)?;
        if verbose < 2 {
            return Ok(())
        }
        write!(f, "\t\tFlags: PMEClk{} DSI{} D1{} D2{} AuxCurrent={} PME(D0{},D1{},D2{},D3hot{},D3cold{})\n",
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
        write!(f, "\t\tStatus: D{} NoSoftRst{} PME-Enable{} DSel={} DScale={} PME{}\n",
            ctrl.power_state as usize,
            ctrl.no_soft_reset.display(BoolView::PlusMinus),
            ctrl.pme_enabled.display(BoolView::PlusMinus),
            d.map(|d| d.select as usize).unwrap_or(0),
            d.map(|d| d.scale as usize).unwrap_or(0),
            ctrl.pme_status.display(BoolView::PlusMinus),
        )?;
        if br.bpcc_enabled || br.b2_b3 || br.reserved != 0 {
            write!(f, "\t\tBridge: PM{} B3{}\n",
                br.bpcc_enabled.display(BoolView::PlusMinus),
                br.b2_b3.display(BoolView::PlusMinus),
            )?;
        }
        Ok(())
    }
}
impl<'a> DisplayMultiView<'a> for PowerManagementInterface {
    type Data = &'a PowerManagementInterface;
    type View = Verbose;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}


#[derive(Debug, Default, PartialEq, Eq)]
pub struct VendorSpecificView {
    vendor_id: u16,
    device_id: u16,
    verbose: usize,
}

impl<'a> DisplayMultiView<'a> for VendorSpecific<'a> {
    type Data = &'a VendorSpecific<'a>;
    type View = &'a VendorSpecificView;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}
impl<'a> fmt::Display for MultiView<&'a VendorSpecific<'a>, &'a VendorSpecificView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use vendor_specific::{VendorCapabilty, Virtio};
        write!(f, "Vendor Specific Information: ")?;
        let vc = VendorCapabilty::new(
            &self.data.0,
            self.view.vendor_id,
            self.view.device_id
        );
        match vc {
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
                    write!(f, "\n")
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
            vendor_specific::VendorCapabilty::Unspecified(slice) =>
                write!(f, "Len={:02x} <?>\n", slice.len() + 1),
        }
    }
}




#[cfg(test)]
mod tests {
    use crate::device::ECS_OFFSET;
    use crate::device::DDR_OFFSET;
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
            \t\tBridge: PM- B3+\n\
        ";
        assert_eq!(v2_sample, v2_result, "-vv");

        let v3_result = pmi.display(Verbose(3)).to_string();
        let v3_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3+\n\
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
        let view = VendorSpecificView {
            verbose: 3,
            vendor_id: 0x1af4,
            device_id: 0x1045,
        };
        assert_eq!(
            "Vendor Specific Information: VirtIO: Notify\n\
            \t\tBAR=4 offset=00003000 size=00001000 multiplier=00000004\n",
            vs.display(&view).to_string()
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
        let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/config"));
        let ddr = data[DDR_OFFSET..ECS_OFFSET].try_into().unwrap();
        let caps = Capabilities::new(&ddr, 0x50);
        let view = lspci::BasicView { verbose: 3, ..Default::default() };
        let s = caps.map(|c| format!("{}", c.display(&view)))
            .collect::<String>();
        let result = s.lines()
            .collect::<Vec<_>>();
        let sample = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/caps.lspci.vvv.txt"))
            .lines().collect::<Vec<_>>();
        assert_eq!(sample, result);
    }
}
