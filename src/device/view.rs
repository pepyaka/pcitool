use std::fmt;

use bitfield_layout::BitFieldLayout;

use super::{Device,HeaderCommon,Header,HeaderTypeNormal,BaseAddress,};
use crate::pciids::PciidsDevices;

pub struct LspciView<'a> {
    device: &'a Device,
    pciids_devices: &'a PciidsDevices,
    /// Machine readable format
    ///
    /// 1. Simple
    /// 2. Verbose
    pub machine: u64,
    /// Verbose
    ///
    /// 1. Be verbose and display detailed information
    /// 2. Be very verbose and display more details. This level includes everything deemed useful
    /// 3. Be  even more verbose and display everything we are able to parse, even if it doesn't
    ///    look interesting at all
    pub verbose: u64,
    /// Show kernel drivers handling and also kernel modules capable of handling it
    pub kernel: bool,
    /// Bus-centric view.
    ///
    /// Show all IRQ numbers and addresses as seen by the cards on the PCI bus instead of as seen
    /// by the kernel.
    pub bus_centric: bool,
    /// Always show PCI domain numbers
    pub always_domain_number: bool,
    /// Identify PCI devices by path through each bridge
    ///
    /// 1. Instead of by bus number
    /// 2. Showing the bus number as well as the device number
    pub path_through: u64,
    /// Show PCI vendor and device codes as numbers
    ///
    /// 1. Instead of looking them up in the PCI ID list
    /// 2. Both numbers and names
    pub as_numbers: u64,
}

impl<'a> LspciView<'a> {
    pub fn new(device: &'a Device, pciids_devices: &'a PciidsDevices) -> Self {
        LspciView {
            device, pciids_devices,
            machine: 0,
            verbose: 0,
            kernel: false,
            bus_centric: false,
            always_domain_number: false,
            path_through: 0,
            as_numbers: 0,
        }
    }
    fn fmt_terse(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Device address
        write!(f, "{:#}", self.device.address)?;
        let HeaderCommon {
            vendor_id,
            device_id,
            // command,
            // status,
            revision_id,
            class_code,
            //cache_line_size,
            // latency_timer,
            //_header_type,
            // bist,
            ..
        } = self.device.header.common();
        match (self.as_numbers, class_code.meaning()) {
            (0, (_, Some(sub_class), _)) => write!(f, " {}:", sub_class)?,
            (0, (class, None, _)) => write!(f, " {}:", class)?,
            (1, (_, _, _)) => write!(f, " {:02x}{:02x}:", class_code.base, class_code.sub)?,
            (_, (_, Some(sub_class), _)) => 
                write!(f, " {} [{:02x}{:02x}]:", sub_class, class_code.base, class_code.sub)?,
            (_, (class, None, _)) => 
                write!(f, " {} [{:02x}{:02x}]:", class, class_code.base, class_code.sub)?,
        }
        let pciids_vendor = self.pciids_devices.vendors.get(&vendor_id);
        let pciids_device = self.pciids_devices.devices.get(&(*vendor_id, *device_id));
        match (self.as_numbers, pciids_vendor, pciids_device) {
            (0, Some(v), Some(d)) => write!(f, " {} {}", v, d)?,
            (0, Some(v), None) => write!(f, " {}", v)?,
            (1, _, _) => write!(f, " {:04x}:{:04x}", vendor_id, device_id)?,
            (_, Some(v), Some(d)) => write!(f, " {} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id)?,
            (_, Some(v), None) => write!(f, " {} [{:04x}:{:04x}]", v, vendor_id, device_id)?,
            _ => write!(f, " [{:04x}:{:04x}]", vendor_id, device_id)?,
        }
        if revision_id != &0 {
            write!(f, " (rev {:02x})", revision_id)?;
        }
        if self.verbose > 0 {
            if let (_, _, Some(prog_if)) = class_code.meaning() {
                write!(f, " (prog-if {:02x}[{}])\n", class_code.interface, prog_if)?;
            } else {
                write!(f, " (prog-if {:02x})\n", class_code.interface)?;
            };
        }
        if self.verbose > 0 || self.kernel {
            if let Some(label) = &self.device.label {
                write!(f, "\tDeviceName: {}", label)?;
            }
            // Subdevice
            if let Header::Normal(
                HeaderTypeNormal {
                    sub_vendor_id: vendor_id, 
                    sub_device_id: device_id,
                    ..
                }) = self.device.header 
            {
                let pciids_sub_vendor = self.pciids_devices.vendors
                    .get(&vendor_id);
                let pciids_sub_device = self.pciids_devices.sub_devices
                    .get(&(vendor_id, device_id));
                match (self.as_numbers, pciids_sub_vendor, pciids_sub_device) {
                    (0, Some(v), Some(d)) =>
                        write!(f, "\tSubsystem: {} {}\n", v, d)?,
                    (0, Some(v), None) =>
                        //  There is mysterious `device_id + 5` in libpci
                        write!(f, "\tSubsystem: {} Device {:04x}\n", v, device_id)?,
                    (0, None, _) =>
                        write!(f, "\tSubsystem: Device {:04x}\n", device_id)?,
                    (1, _, _) =>
                        write!(f, "\tSubsystem: {}:{}\n", vendor_id, device_id)?,
                    (_, Some(v), Some(d)) => 
                        write!(f, "\tSubsystem: {} {} [{:04x}:{:04x}]\n", v, d, vendor_id, device_id)?,
                    (_, Some(v), None) =>
                        write!(f, "\tSubsystem: {} Device [{:04x}:{:04x}]\n", v, vendor_id, device_id)?,
                    (_, None, _) =>
                        write!(f, "\tSubsystem: Device [{:04x}:{:04x}]\n", vendor_id, device_id)?,
                }
            };
        }
        Ok(())
    }
    fn fmt_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let HeaderCommon {
            // vendor_id,
            // device_id,
            command,
            status,
            // revision_id,
            // class_code,
            // cache_line_size,
            latency_timer,
            // _header_type,
            bist,
            ..
        } = self.device.header.common();
        if let Some(phy_slot) = &self.device.phy_slot {
            write!(f, "\tPhysical Slot: {}\n", phy_slot)?;
        }

        // TODO: Device tree node

        if self.verbose > 1 {
            todo!()
        } else {
            write!(f, "\tFlags: ")?;
            if let Some(true) = command.find_state(|v| v.lspci == "BusMaster") {
                write!(f, "bus master, ")?;
            }
            if let Some(true) = command.find_state(|v| v.lspci == "VGASnoop") {
                write!(f, "VGA palette snoop, ")?;
            }
            if let Some(true) = command.find_state(|v| v.lspci == "Stepping") {
                write!(f, "stepping, ")?;
            }
            if let Some(true) = command.find_state(|v| v.lspci == "FastB2B") {
                write!(f, "fast Back2Back, ")?;
            }
            if let Some(true) = status.find_state(|v| v.lspci == "66MHz") {
                write!(f, "66MHz, ")?;
            }
            if let Some(true) = status.find_state(|v| v.lspci == "UDF") {
                write!(f, "user-definable features, ")?;
            }
            let devsel_medium = status.find_state(|v| v.lspci == "DEVSEL=medium");
            let devsel_slow = status.find_state(|v| v.lspci == "DEVSEL=slow");
            match (devsel_medium, devsel_slow) {
                (Some(false), Some(false)) => write!(f, "fast devsel")?,
                (Some(true), Some(false)) => write!(f, "medium devsel")?,
                (Some(false), Some(true)) => write!(f, "slow devsel")?,
                _ => write!(f, "??")?,
            }
            if let Some(true) = command.find_state(|v| v.lspci == "BusMaster") {
                write!(f, ", latency {}", latency_timer)?;
            }
            let irq = self.device.irq();
            if irq > 0 {
                #[cfg(target_arch = "sparc64")]
                write!(f, ", IRQ {:08x}", irq)?;
                #[cfg(not (target_arch = "sparc64"))]
                write!(f, ", IRQ {}", irq)?;
            }
            if let Some(numa_node) = &self.device.numa_node {
                write!(f, ", NUMA node {}", numa_node)?;
            }
            if let Some(iommu_group) = &self.device.iommu_group {
                write!(f, ", IOMMU group {}", iommu_group)?;
            }
            write!(f, "\n")?;
        }
        // BIST
        if bist.is_capable {
            if bist.is_running {
                write!(f, "\tBIST is running\n")?;
            } else {
                write!(f, "\tBIST result: {:02x}\n", bist.completion_code)?;
            }
        }
        match &self.device.header {
            Header::Normal(_) => self.fmt_header_normal(f),
            Header::Bridge(_) => self.fmt_header_bridge(f),
            Header::Cardbus(_) => self.fmt_header_cardbus(f),
        }
    }
    // ref to show_htype0(struct device *d);
    fn fmt_header_normal(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_bases(f)?;
        self.fmt_rom(f)?;
        self.fmt_capabilities(f)
    }
    // ref to show_htype1(struct device *d);
    fn fmt_header_bridge(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_bases(f)?;
        // TODO:
        // printf("\tBus: primary=%02x, secondary=%02x, subordinate=%02x, sec-latency=%d\n",
        // show_range("\tI/O behind bridge", io_base, io_limit+0xfff, 0);
        // show_range("\tMemory behind bridge", mem_base, mem_limit + 0xfffff, 0);
        // show_range("\tPrefetchable memory behind bridge", pref_base_64, pref_limit_64 + 0xfffff, (pref_type == PCI_PREF_RANGE_TYPE_64));
        if self.verbose > 1 {
            // TODO:
            // printf("\tSecondary status: 66MHz%c FastB2B%c ParErr%c DEVSEL=%s >TAbort%c <TAbort%c <MAbort%c <SERR%c <PERR%c\n",
        }
        self.fmt_rom(f)?;
        if self.verbose > 1 {
            // TODO:
            // printf("\tBridgeCtl: Parity%c SERR%c NoISA%c VGA%c VGA16%c MAbort%c >Reset%c FastB2B%c\n",
            // printf("\t\tPriDiscTmr%c SecDiscTmr%c DiscTmrStat%c DiscTmrSERREn%c\n",
        }
        self.fmt_capabilities(f)
    }
    // ref to show_htype2(struct device *d);
    fn fmt_header_cardbus(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_bases(f)?;
        // TODO:
        // printf("\tBus: primary=%02x, secondary=%02x, subordinate=%02x, sec-latency=%d\n",
        // printf("\tMemory window %d: %08x-%08x%s%s\n", i, base, limit,
        // printf("\tI/O window %d: %08x-%08x%s\n", i, base, limit,
        // printf("\tSecondary status: SERR\n");
        // if (get_conf_word(d, PCI_CB_SEC_STATUS) & PCI_STATUS_SIG_SYSTEM_ERROR)
        //    printf("\tSecondary status: SERR\n");
        if self.verbose > 1 {
            // TODO:
            // printf("\tBridgeCtl: Parity%c SERR%c ISA%c VGA%c MAbort%c >Reset%c 16bInt%c PostWrite%c\n",
        }
        // TODO:
        // if (exca)
        //    printf("\t16-bit legacy interface ports at %04x\n", exca);
        self.fmt_capabilities(f)
    }
    fn fmt_bases(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ba in self.device.header.base_addresses().iter() {
            match ba {
                BaseAddress::MemorySpace32 { base_address, prefetchable, size, .. } => {
                    let pf = if prefetchable { "prefetchable" } else { "non-prefetchable" };
                    write!(f, "\tMemory at {:x} (32-bit, {})", base_address, pf)?;
                    if let Some(s) = size { fmt_size(f, s as u64)?; };
                },
                BaseAddress::MemorySpace64 { base_address, prefetchable, size, .. } => {
                    let pf = if prefetchable { "prefetchable" } else { "non-prefetchable" };
                    write!(f, "\tMemory at {:x} (64-bit, {})", base_address, pf)?;
                    if let Some(s) = size { fmt_size(f, s)?; };
                },
                BaseAddress::IoSpace { base_address, .. } => {
                    write!(f, "\tMemory at {:x}", base_address)?;
                },
                BaseAddress::Reserved => {
                    write!(f, "\tMemory at <broken-64-bit-slot>")?;
                },
            }
            write!(f, "\n")?;

        }
        Ok(())
    }
    fn fmt_machine(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TODO: fmt_machine")
    }
    fn fmt_kernel(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kernel {
           write!(f, "\tTODO: fmt_kernel")?;
        }
        Ok(())
    }
    fn fmt_rom(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(rom) = self.device.header.expansion_rom() {
            if rom.address != 0 {
                write!(f, "\tTODO: fmt_rom")?;
            }
        }
        Ok(())
    }
    fn fmt_capabilities(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let caps = caps
        //     .map(|v| v.to_string()).collect::<String>();
        // write!(f, "{}", caps)
        Ok(())
    }

}
impl<'a> fmt::Display for LspciView<'a> {
    // refer to show_device(struct device *d);
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.machine > 0 {
            self.fmt_machine(f)
        } else {
            match (self.verbose, self.kernel) {
                (0, false) => self.fmt_terse(f),
                (0, true) => {
                    self.fmt_terse(f)?;
                    self.fmt_kernel(f)
                },
                _ => {
                    self.fmt_terse(f)?;
                    self.fmt_verbose(f)?;
                    self.fmt_kernel(f)
                },
            }
        }
    }
}


fn fmt_size(f: &mut fmt::Formatter<'_>, x: u64) -> fmt::Result {
    let suffix = [ "", "K", "M", "G", "T" ];
    if x == 0 {
        Ok(())
    } else {
        let mut i = 0;
        let mut result = x;
        while (result % 1024 == 0) && (i < suffix.len()) {
            result /= 1024;
            i += 1;
        }
        write!(f, " [size={}{}]", result, suffix[i])
    }
}

// fn fmt_range(f: &mut fmt::Formatter<'_>, base: u64, limit: u64, is_64bit: bool, verbose: u64) -> fmt::Result {
//     if base <= limit || verbose > 2 {
//         if is_64bit {
//             write!(f, " {:016x}-{:016x}", base, limit)?;
//         } else {
//             write!(f, " {:08x}-{:08x}", base, limit)?;
//         }
//     }
//     if base <= limit {
//         fmt_size(f, limit - base + 1)?;
//     }  else {
//         write!(f, " [disabled]\n")?;
//     }
//     write!(f, "\n")
// }

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::convert::{TryFrom, TryInto};
    use crate::{device::{Device, Header, BaseAddressSized,BaseAddresses}, pciids, };
    use super::*;

    #[test]
    fn base() {
        // Audio device: Intel Corporation Cannon Point-LP High Definition Audio Controller (rev 30) (prog-if 80)
        // Subsystem: ASUSTeK Computer Inc. Device 16a1
        // Control: I/O- Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx+
        // Status: Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
        // Latency: 32, Cache Line Size: 64 bytes
        // Interrupt: pin A routed to IRQ 145
        // Region 0: Memory at b4418000 (64-bit, non-prefetchable) [size=16K]
        // Region 4: Memory at b4100000 (64-bit, non-prefetchable) [size=1M]
        // Capabilities: [50] Power Management version 3
        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=55mA PME(D0-,D1-,D2-,D3hot+,D3cold+)
        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
        // Capabilities: [80] Vendor Specific Information: Len=14 <?>
        // Capabilities: [60] MSI: Enable+ Count=1/1 Maskable- 64bit+
        //         Address: 00000000fee00578  Data: 0000
        // Kernel driver in use: snd_hda_intel
        // Kernel modules: snd_hda_intel, snd_soc_skl, snd_sof_pci_intel_cnl
        let pciids_devices = 
            pciids::PciDevices::new(include_str!("/usr/share/hwdata/pci.ids"))
                .collect();
        let data = include_bytes!("../../tests/data/device/8086:9dc8/config");
        let mut header: Header = Header::try_from(&data[..]).unwrap();
        if let Header::Normal(ref mut header) = header {
            if let BaseAddresses::Basic(bars) = header.base_addresses {
                let sized: [BaseAddressSized; 6] = bars.iter()
                    .map(|&dword| {
                        match dword {
                            0xb4418004 => BaseAddressSized { data: dword, size: 16 << 10 },
                            0xb4100004 => BaseAddressSized { data: dword, size: 1 << 20 },
                            _ => BaseAddressSized { data: dword, size: 0 },
                        }
                    })
                    .collect::<Vec<_>>().try_into().unwrap();
                header.base_addresses = BaseAddresses::Sized(sized);
            }
                //.iter().map(|bar| bar).collect::<Vec<_>>().try_into().unwrap();
            // if let BaseAddresses::Basic(bars) = header.base_addresses {
            // }
        }
        let mut device = Device::new("00:1f.3".parse().unwrap(), header);
        device.irq = Some(145);

        println!("{:#x?}", &device.header);
        
        let mut view = LspciView::new(&device, &pciids_devices);
        assert_eq!(
            "00:1f.3 Audio device: Intel Corporation Cannon Point-LP High Definition Audio Controller (rev 30)",
            view.to_string(),
            "Args: -"
        );
        
        view.as_numbers = 1;
        assert_eq!(
            "00:1f.3 0403: 8086:9dc8 (rev 30)",
            view.to_string(),
            "Args: -n"
        );
        
        view.as_numbers = 2;
        assert_eq!(
            "00:1f.3 Audio device [0403]: Intel Corporation Cannon Point-LP High Definition Audio Controller [8086:9dc8] (rev 30)",
            view.to_string(),
            "Args: -nn"
        );

        view.as_numbers = 0;
        view.verbose = 1;
        assert_eq!(
            include_str!("../../tests/data/device/8086:9dc8/args.v.txt"),
            view.to_string(),
            "Args: -nn"
        );
    }
}

