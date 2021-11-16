#[cfg(feature = "clap")]
use clap::Clap;


use crate::device::header::{BridgeIoAddressRange, BridgePrefetchableMemory};
use crate::device::header::bar::IoResource;
use crate::{
    device::{
        self,
        Device,
        header::{
            self,
            Command,
            bar::BaseAddressMap,
            HeaderType,
            InterruptPin
        }
    },
    pciids::VendorDeviceSubsystem,
};
use crate::view::*;

mod caps;

#[cfg(feature = "kmod")]
mod kmod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspciDevice<'a> {
    pub device: &'a Device,
    pub vds: &'a VendorDeviceSubsystem,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(Clap))]
pub struct BasicView {
    /// Be verbose (-vv or -vvv for higher verbosity)
    #[cfg_attr(feature = "clap", clap(short = 'v', parse(from_occurrences)))]
    pub verbose: usize,
    /// Show kernel drivers handling each device
    #[cfg_attr(feature = "clap", clap(short = 'k'))]
    pub kernel: bool,
    /// Bus-centric view (addresses and IRQ's as seen by the bus)
    #[cfg_attr(feature = "clap", clap(short = 'b'))]
    pub bus_centric: bool,
    /// Always show domain numbers
    #[cfg_attr(feature = "clap", clap(short = 'D'))]
    pub always_domain_number: bool,
    /// Display bridge path in addition to bus and device number
    #[cfg_attr(feature = "clap", clap(short = 'P', parse(from_occurrences)))]
    pub path_through: usize,
    /// Show numeric ID's
    #[cfg_attr(feature = "clap", clap(short = 'n', parse(from_occurrences)))]
    pub as_numbers: usize,
}


impl<'a> DisplayMultiView<'a> for LspciDevice<'a> {
    type Data = &'a LspciDevice<'a>;
    type View = &'a BasicView;
    fn display(&'a self, view: Self::View) -> MultiView<Self::Data, Self::View> {
        MultiView { data: self, view, }
    }
}

impl<'a> fmt::Display for MultiView<&'a LspciDevice<'a>, &'a BasicView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.view.verbose > 0 {
            self.fmt_terse(f)?;
            self.fmt_verbose(f)?;
            writeln!(f, "")
        } else {
            self.fmt_terse(f)
        }
    }
}

impl<'a> MultiView<&'a LspciDevice<'a>, &'a BasicView> {
    fn fmt_terse(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let view = self.view;
        let LspciDevice {
            device,
            vds,
        } = self.data;
        // Device address
        if view.always_domain_number {
            write!(f, "{:}", device.address)?;
        } else {
            write!(f, "{:#}", device.address)?;
        }
        let device::Header {
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
        } = &device.header;

        // PCI_LOOKUP_CLASS
        match (view.as_numbers, class_code.meaning()) {
            (0, (_, Some(sub_class), _)) => write!(f, " {}:", sub_class)?,
            (0, ("Reserved", None, _)) => write!(f, " Class [{:02x}{:02x}]:", class_code.base, class_code.sub)?,
            (0, (class, None, _)) => write!(f, " {} [{:02x}{:02x}]:", class, class_code.base, class_code.sub)?,
            // Args: -n
            (1, (_, _, _)) => write!(f, " {:02x}{:02x}:", class_code.base, class_code.sub)?,
            // Args: -nn+G
            (_, (_, Some(sub_class), _)) => 
                write!(f, " {} [{:02x}{:02x}]:", sub_class, class_code.base, class_code.sub)?,
            (_, ("Reserved", _, _)) => 
                write!(f, " Class [{:02x}{:02x}]:", class_code.base, class_code.sub)?,
            (_, (class, None, _)) => 
                write!(f, " {} [{:02x}{:02x}]:", class, class_code.base, class_code.sub)?,
        }
        // TODO: Long string ellipsis:
        // if (res >= size && size >= 4)
        //   buf[size-2] = buf[size-3] = buf[size-4] = '.';
        // else if (res < 0 || res >= size)
        //   return "<pci_lookup_name: buffer too small>";

        // PCI_LOOKUP_VENDOR | PCI_LOOKUP_DEVICE
        let pciids_vendor = vds.lookup(*vendor_id, None, None);
        let pciids_device = vds.lookup(*vendor_id, *device_id, None);
        match (view.as_numbers, pciids_vendor, pciids_device) {
            (0, Some(v), Some(d)) => write!(f, " {} {}", v, d)?,
            (0, Some(v), None) => write!(f, " {} Device {:04x}", v, device_id)?,
            (1, _, _) => write!(f, " {:04x}:{:04x}", vendor_id, device_id)?,
            (_, Some(v), Some(d)) => write!(f, " {} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id)?,
            (_, Some(v), None) => write!(f, " {} Device [{:04x}:{:04x}]", v, vendor_id, device_id)?,
            _ => write!(f, " [{:04x}:{:04x}]", vendor_id, device_id)?,
        }

        // PCI_REVISION_ID
        if revision_id != &0 {
            write!(f, " (rev {:02x})", revision_id)?;
        }

        // PCI_LOOKUP_PROGIF | PCI_LOOKUP_NO_NUMBERS
        if view.verbose > 0 {
            match (class_code.interface, class_code.meaning()) {
                (0, (_, _, None)) => (),
                (_, (_, _, Some(prog_if))) =>
                    write!(f, " (prog-if {:02x} [{}])", class_code.interface, prog_if)?,
                _ =>
                    write!(f, " (prog-if {:02x})", class_code.interface)?,
            };
        }
        writeln!(f, "")?;

        if view.verbose > 0 || view.kernel {
            // PCI_FILL_LABEL
            if let Some(label) = &device.label {
                write!(f, "\tDeviceName: {}", label)?;
            }
            // Subdevice
            if let device::HeaderType::Normal(header::Normal { sub_vendor_id, sub_device_id, ..  })
                = device.header.header_type
            {
                let pciids_sub_vendor = vds.lookup(sub_vendor_id, None, None);
                let pciids_sub_device = vds.lookup(*vendor_id, *device_id, (sub_vendor_id, sub_device_id));

                // PCI_LOOKUP_SUBSYSTEM | PCI_LOOKUP_VENDOR | PCI_LOOKUP_DEVICE
                match (view.as_numbers, pciids_sub_vendor, pciids_sub_device) {
                    (0, Some(v), Some(d)) =>
                        write!(f, "\tSubsystem: {} {}\n", v, d)?,
                    (0, Some(v), None) =>
                        //  There is mysterious `device_id + 5` in libpci
                        write!(f, "\tSubsystem: {} Device {:04x}\n", v, sub_device_id)?,
                    (0, None, _) =>
                        write!(f, "\tSubsystem: Device {:04x}\n", sub_device_id)?,
                    (1, _, _) =>
                        write!(f, "\tSubsystem: {:04x}:{:04x}\n", sub_vendor_id, sub_device_id)?,
                    (_, Some(v), Some(d)) => 
                        write!(f, "\tSubsystem: {} {} [{:04x}:{:04x}]\n", v, d, sub_vendor_id, sub_device_id)?,
                    (_, Some(v), None) =>
                        write!(f, "\tSubsystem: {} Device [{:04x}:{:04x}]\n", v, sub_vendor_id, sub_device_id)?,
                    (_, None, _) =>
                        write!(f, "\tSubsystem: Device [{:04x}:{:04x}]\n", sub_vendor_id, sub_device_id)?,
                }
            };
        }
        Ok(())
    }
    fn fmt_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let view = self.view;
        let LspciDevice {
            device: Device {
                header: device::Header {
                    command,
                    status,
                    cache_line_size,
                    latency_timer,
                    header_type,
                    bist,
                    interrupt_pin,
                    ..
                },
                phy_slot,
                numa_node,
                iommu_group,
                ..
            },
            ..
        } = self.data;
        if let Some(phy_slot) = &phy_slot {
            write!(f, "\tPhysical Slot: {}\n", phy_slot)?;
        }

        // TODO: Device tree node

        let irq = self.data.device.irq();
        if view.verbose > 1 {
            write!(f, "\tControl: {}\n\tStatus: {}\n",
                command.display(&View::LspciBasic(*view)),
                status.display(&View::LspciBasic(*view)),
            )?;
            if command.bus_master {
                write!(f, "\tLatency: {}", latency_timer)?;
                if let HeaderType::Normal(header::Normal { min_grant, max_latency, .. }) = header_type {
                    match (*min_grant as u16 * 250, *max_latency as u16 * 250) {
                        (0, 0) => Ok(()),
                        (min, 0) => write!(f, " ({}ns min)", min),
                        (0, max) => write!(f, " ({}ns max)", max),
                        (min, max) => write!(f, " ({}ns min, {}ns max)", min, max),
                    }?;
                }
                if cache_line_size != &0 {
                    write!(f, ", Cache Line Size: {} bytes", cache_line_size * 4)?;
                }
                write!(f, "\n")?;
            }
            match interrupt_pin {
                InterruptPin::Unused if irq != 0 =>
                    write!(f, "\tInterrupt: pin ? routed to IRQ {}\n", irq)?,
                InterruptPin::IntA =>
                    write!(f, "\tInterrupt: pin A routed to IRQ {}\n", irq)?,
                InterruptPin::IntB =>
                    write!(f, "\tInterrupt: pin B routed to IRQ {}\n", irq)?,
                InterruptPin::IntC =>
                    write!(f, "\tInterrupt: pin C routed to IRQ {}\n", irq)?,
                InterruptPin::IntD =>
                    write!(f, "\tInterrupt: pin D routed to IRQ {}\n", irq)?,
                InterruptPin::Reserved(v) => {
                    if let Some(pin) = char::from_u32(('A' as u32) + (u8::from(*v) as u32) - 1) {
                        write!(f, "\tInterrupt: pin {} routed to IRQ {}\n", pin, irq)?;
                    };
                },
                _ => (),
            };
            if let Some(numa_node) = numa_node {
                write!(f, "\tNUMA node: {}\n", numa_node)?;
            }
            if let Some(iommu_group) = iommu_group {
                write!(f, "\tIOMMU group: {}\n", iommu_group)?;
            }
        } else {
            write!(f, "\tFlags: {}{}{}{}{}{}{}",
                command.bus_master.display(BoolView::Str("bus master, ")),
                command.vga_palette_snoop.display(BoolView::Str("VGA palette snoop, ")),
                command.stepping.display(BoolView::Str("stepping, ")),
                command.fast_back_to_back_enable.display(BoolView::Str("fast Back2Back, ")),
                status.is_66mhz_capable.display(BoolView::Str("66MHz, ")),
                status.user_definable_features.display(BoolView::Str("user-definable features, ")),
                format!("{} devsel", status.devsel_timing),
            )?;
            if command.bus_master {
                write!(f, ", latency {}", latency_timer)?;
            }
            if irq != 0 {
                #[cfg(target_arch = "sparc64")]
                write!(f, ", IRQ {:08x}", irq)?;
                #[cfg(not (target_arch = "sparc64"))]
                write!(f, ", IRQ {}", irq)?;
            }
            if let Some(numa_node) = numa_node {
                write!(f, ", NUMA node {}", numa_node)?;
            }
            if let Some(iommu_group) = iommu_group {
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
        match header_type {
            device::HeaderType::Normal(_) => self.fmt_header_normal(f),
            device::HeaderType::Bridge(bridge) => self.fmt_header_bridge(f, bridge),
            device::HeaderType::Cardbus(_) => self.fmt_header_cardbus(f),
        }
    }
    // ref to show_htype0(struct device *d);
    fn fmt_header_normal(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_bases(f)?;
        self.fmt_rom(f)?;
        self.fmt_capabilities(f)
    }
    // ref to show_htype1(struct device *d);
    fn fmt_header_bridge(&self, f: &mut fmt::Formatter<'_>, bridge: &header::Bridge) -> fmt::Result {
        let verbose = self.view.verbose as u64;
        let header::Bridge {
            primary_bus_number,
            secondary_bus_number,
            subordinate_bus_number,
            secondary_latency_timer,
            io_address_range,
            memory_base,
            memory_limit,
            prefetchable_memory,
            secondary_status,
            bridge_control,
            ..
        }  = bridge;
        self.fmt_bases(f)?;
        write!(f, "\tBus: primary={:02x}, secondary={:02x}, subordinate={:02x}, sec-latency={}\n",
            primary_bus_number, secondary_bus_number, subordinate_bus_number, secondary_latency_timer
        )?;
        match io_address_range {
            BridgeIoAddressRange::NotImplemented =>
                write!(f, "\tI/O behind bridge: [disabled]")?,
            BridgeIoAddressRange::IoAddr16 { base, limit } => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfff, false, verbose)?
            },
            BridgeIoAddressRange::IoAddr32 { base, limit } => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfff, false, verbose)?
            },
            BridgeIoAddressRange::Reserved { base, limit } =>
                write!(f, "\t!!! Unknown I/O range types {:x}/{:x}\n", base, limit)?,
        }
        if (memory_base & 0xf) != 0 && (memory_limit & 0xf) != 0 {
            write!(f, "\t!!! Unknown memory range types {:x}/{:x}\n", memory_base, memory_limit)?;
        } else {
            let memory_base = ((memory_base & !0xf) as u64) << 16;
            let memory_limit = ((memory_limit & !0xf) as u64) << 16;
            write!(f, "\tMemory behind bridge:")?;
            fmt_range(f, memory_base, memory_limit + 0xfffff, false, verbose)?;
        }
        match prefetchable_memory {
            BridgePrefetchableMemory::NotImplemented =>
                write!(f, "\tPrefetchable memory behind bridge: [disabled]")?,
            BridgePrefetchableMemory::MemAddr32 { base, limit } => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfffff, false, verbose)?
            },
            BridgePrefetchableMemory::MemAddr64 { base, limit } => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, *base, *limit + 0xfffff, true, verbose)?
            },
            BridgePrefetchableMemory::Reserved { base, limit } =>
                write!(f, "\t!!! Unknown prefetchable memory range types {:x}/{:x}\n", base, limit)?,
        }
        if verbose > 1 {
            write!(f, "\tSecondary status: {}\n", secondary_status.display(&View::LspciBasic(*self.view)))?;
        }
        self.fmt_rom(f)?;
        if verbose > 1 {
            write!(f,
                "\tBridgeCtl: Parity{} SERR{} NoISA{} VGA{} VGA16{} MAbort{} >Reset{} FastB2B{}\n",
                bridge_control.parity_error_response_enable.display(BoolView::PlusMinus),
                bridge_control.serr_enable.display(BoolView::PlusMinus),
                bridge_control.isa_enable.display(BoolView::PlusMinus),
                bridge_control.vga_enable.display(BoolView::PlusMinus),
                bridge_control.vga_16_enable.display(BoolView::PlusMinus),
                bridge_control.master_abort_mode.display(BoolView::PlusMinus),
                bridge_control.secondary_bus_reset.display(BoolView::PlusMinus),
                bridge_control.fast_back_to_back_enable.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\tPriDiscTmr{} SecDiscTmr{} DiscTmrStat{} DiscTmrSERREn{}\n",
                bridge_control.primary_discard_timer.display(BoolView::PlusMinus),
                bridge_control.secondary_discard_timer.display(BoolView::PlusMinus),
                bridge_control.discard_timer_status.display(BoolView::PlusMinus),
                bridge_control.discard_timer_serr_enable.display(BoolView::PlusMinus),
            )?;
        }
        self.fmt_capabilities(f)
    }
    // ref to show_htype2(struct device *d);
    fn fmt_header_cardbus(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.verbose;
        self.fmt_bases(f)?;
        // TODO:
        // printf("\tBus: primary=%02x, secondary=%02x, subordinate=%02x, sec-latency=%d\n",
        // printf("\tMemory window %d: %08x-%08x%s%s\n", i, base, limit,
        // printf("\tI/O window %d: %08x-%08x%s\n", i, base, limit,
        // printf("\tSecondary status: SERR\n");
        // if (get_conf_word(d, PCI_CB_SEC_STATUS) & PCI_STATUS_SIG_SYSTEM_ERROR)
        //    printf("\tSecondary status: SERR\n");
        if verbose > 1 {
            // TODO:
            // printf("\tBridgeCtl: Parity%c SERR%c ISA%c VGA%c MAbort%c >Reset%c 16bInt%c PostWrite%c\n",
        }
        // TODO:
        // if (exca)
        //    printf("\t16-bit legacy interface ports at %04x\n", exca);
        self.fmt_capabilities(f)
    }
    fn fmt_bases(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Command { io_space, memory_space, .. } = self.data.device.header.command;
        let pf = |prefetchable: bool| if prefetchable { "" } else { "non-" };
        for ba in self.data.device.header.header_type.base_addresses() {
            write!(f, "\t")?;
            if self.view.verbose > 1 {
                write!(f, "Region {}: ", ba.region)?;
            }

            let (is_zero_ba, is_disabled, size) = match ba.map {
                BaseAddressMap::IoSpace { base_address, size } => {
                    if base_address != 0 || io_space {
                        write!(f, "I/O ports at {:x}", base_address)?;
                    } else if size > Some(0) {
                        write!(f, "I/O ports at <ignored>")?;
                    } else {
                        write!(f, "I/O ports at <unassigned>")?;
                    }
                    (base_address == 0, !io_space, size.map(u64::from))
                },
                BaseAddressMap::MemorySpace64Broken => {
                    write!(f, "Memory at <broken-64-bit-slot>")?;
                    (false, !memory_space, None)
                },
                // TODO: <ignored>
                BaseAddressMap::MemorySpace32 { base_address, prefetchable, size, .. } => {
                    match (base_address, memory_space) {
                        (0, false) => write!(f, "Memory at <unassigned>")?,
                        (_, false) => write!(f, "Memory at <ignored>")?,
                        _ => write!(f, "Memory at {:x}", base_address)?,
                    };
                    write!(f, " (32-bit, {}prefetchable)", pf(prefetchable))?;
                    (base_address == 0, !memory_space, size.map(u64::from))
                },
                BaseAddressMap::MemorySpace1M { base_address, prefetchable, size, .. } => {
                    match (base_address, memory_space) {
                        (0, false) => write!(f, "Memory at <unassigned>")?,
                        (_, false) => write!(f, "Memory at <ignored>")?,
                        _ => write!(f, "Memory at {:x}", base_address)?,
                    };
                    write!(f, " (low-1M, {}prefetchable)", pf(prefetchable))?;
                    (base_address == 0, !memory_space, size.map(u64::from))
                },
                BaseAddressMap::MemorySpace64 { base_address, prefetchable, size, .. } => {
                    if base_address != 0 {
                        write!(f, "Memory at {:x}", base_address)?;
                    } else if size > Some(0) {
                        write!(f, "Memory at <ignored>")?;
                    } else {
                        write!(f, "Memory at <unassigned>")?;
                    }
                    // match (base_address, memory_space) {
                    //     (0, false) => write!(f, "Memory at <unassigned>")?,
                    //     (_, false) => write!(f, "Memory at <ignored>")?,
                    //     _ => write!(f, "Memory at {:x}", base_address)?,
                    // };
                    write!(f, " (64-bit, {}prefetchable)", pf(prefetchable))?;
                    (base_address == 0, !memory_space, size)
                },
                BaseAddressMap::Reserved => (false, false, None),
            };
            let is_zero_bei = matches!(
                ba.io_resource,
                None | Some(IoResource { bar_equivalent_indicator: 0, .. })
            );
            // Detect virtual regions, which are reported by the OS, but unassigned in the device
            // (    pos     ) && (!hw_lower && !hw_upper) && !(ioflg & PCI_IORESOURCE_PCI_EA_BEI)
            if is_zero_ba && is_zero_bei {
                write!(f, " [virtual]")?;
            } else if is_disabled {
                write!(f, " [disabled]")?;
            }
            if ba.io_resource.is_some() && !is_zero_bei {
                write!(f, " [enchanced]")?;
            }
            if let Some(s) = size { 
                fmt_size(f, s as u64)?; 
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
    // fn fmt_machine(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //     write!(f, "TODO: fmt_machine")
    // }
    fn fmt_rom(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data.device.header.header_type {
            device::HeaderType::Normal(header::Normal { expansion_rom, .. }) => {
                if expansion_rom.address != 0 {
                    write!(f, "\tTODO: fmt_rom")?;
                }
            },
            device::HeaderType::Bridge(header::Bridge { expansion_rom, .. }) => {
                if expansion_rom.address != 0 {
                    write!(f, "\tTODO: fmt_rom")?;
                }
            },
            _ => (),
        };
        Ok(())
    }
    fn fmt_capabilities(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.device.header.status.capabilities_list {
            if let Some(caps) = self.data.device.capabilities() {
                for cap in caps {
                    write!(f, "{}", cap.display(&self.view))?;
                }
            } else {
                write!(f, "\tCapabilities: <access denied>\n")?;
            }
        }
        Ok(())
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

fn fmt_range(f: &mut fmt::Formatter<'_>, base: u64, limit: u64, is_64bit: bool, verbose: u64) -> fmt::Result {
    if base <= limit || verbose > 2 {
        if is_64bit {
            write!(f, " {:016x}-{:016x}", base, limit)?;
        } else {
            write!(f, " {:08x}-{:08x}", base, limit)?;
        }
    }
    if base <= limit {
        fmt_size(f, limit - base + 1)?;
    }  else {
        write!(f, " [disabled]")?;
    }
    write!(f, "\n")
}


#[cfg(test)]
mod tests {
    use byte::{
        ctx::*,
        self,
        BytesExt,
    };
    use crate::{
        device::{
            Device,
            ConfigurationSpace,
            address::Address,
            header::bar::{
                BaseAddresses,
                BaseAddressSized
            }
        },
        pciids::PciIds,
    };
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref VDS: VendorDeviceSubsystem = 
            PciIds::new(include_str!("/usr/share/hwdata/pci.ids").lines())
                .collect::<(VendorDeviceSubsystem, _)>().0;
        static ref I9DC8_DEVICE: Device = {
            let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/config"));
            let mut cs: ConfigurationSpace = data.read_with(&mut 0, LE).unwrap();
            // Emulate sized base_addresses
            if let device::HeaderType::Normal(header::Normal { ref mut base_addresses, .. }) = &mut cs.header.header_type {
                let mut sized: [BaseAddressSized; 6] = Default::default();
                if let BaseAddresses::Basic(array) = base_addresses {
                    for (i, &data) in array.iter().enumerate() {
                        let size = match i {
                            0 => 16 << 10,
                            4 => 1 << 20,
                            _ => 0,
                        };
                        sized[i] = BaseAddressSized { data, size };
                    }
                }
                *base_addresses = BaseAddresses::Sized(sized);
            }

            let address: Address = "00:1f.3".parse().unwrap();
            let mut device = Device::new(address, cs);
            device.irq = Some(145);
            device
        };
        static ref I9DC8: LspciDevice<'static> = {
            LspciDevice { device: &I9DC8_DEVICE, vds: &VDS }
        };
    }

    #[test]
    fn display_device_basic() {
        let view = BasicView::default();
        assert_eq!(
            "00:1f.3 Audio device: Intel Corporation Cannon Point-LP High Definition Audio Controller (rev 30)\n",
            I9DC8.display(&view).to_string(),
        );
    }

    mod display_device_as_numbers {
        use super::*;
        macro_rules! display_device_as_numbers {
            ($($id:ident: $sample:expr, $val:expr;)*) => {
                $(
                    #[test]
                    fn $id() {
                        let view = BasicView { as_numbers: $val, ..Default::default() };
                        let result = I9DC8.display(&view).to_string();
                        pretty_assertions::assert_eq!($sample, result);
                    }
                )*
            };
        }
        display_device_as_numbers! {
            eq_1: "00:1f.3 0403: 8086:9dc8 (rev 30)\n", 1;
            eq_2:
                "00:1f.3 Audio device [0403]: Intel Corporation Cannon Point-LP High Definition Audio Controller [8086:9dc8] (rev 30)\n",
                2;
        }
    }
    mod display_device_verbose {
        use super::*;
        macro_rules! display_device_verbose {
            ($($id:ident: $out:expr, $val:expr;)*) => {
                $(
                    #[test]
                    fn $id() {
                        let view = BasicView { verbose: $val, ..Default::default() };
                        let result = I9DC8.display(&view).to_string();
                        let result = result.lines().collect::<Vec<_>>();
                        let sample =
                            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                            "/tests/data/device/", $out)
                            ).lines().collect::<Vec<_>>();
                        pretty_assertions::assert_eq!(sample, result);
                    }
                )*
            };  
        }
        display_device_verbose! {
            eq_1: "8086:9dc8/out.v.txt", 1;
            eq_2: "8086:9dc8/out.vv.txt", 2;
            eq_3: "8086:9dc8/out.vvv.txt", 3;
        }
    }
}

