use core::fmt;

#[cfg(feature = "clap")]
use clap::Parser;

use pcics::{
    header::{
        self, Command, HeaderType, InterruptPin, BridgeIoAddressRange,
        BridgePrefetchableMemory, ClassCode, IoAccessAddressRange, Normal, Bridge, Cardbus,
    },
    capabilities::CapabilityKind, Header
};
use crate::{
    device::{
        self,
        Device,
    },
    pciids,
};
use crate::view::{BoolView, DisplayMultiView, MultiView};
use self::{caps::CapsView, ecaps::EcapsView};


mod hdr;
mod caps;
mod ecaps;

#[cfg(feature = "kmod")]
pub mod kmod;


const PCI_IORESOURCE_PCI_EA_BEI: u64 = 1 << 5;


pub struct Flag(pub bool);

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", super::View::<bool, '±'>(self.0))
    }
}

pub struct View<T>(T);


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspciView {
    pub basic_view: BasicView,
    pub vendor_device_subsystem: pciids::VendorDeviceSubsystem,
    pub class_code: pciids::ClassCode,
}


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(Parser))]
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

impl<'a> DisplayMultiView<&'a LspciView> for Device {}
impl<'a> fmt::Display for MultiView<&'a Device, &'a LspciView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.view.basic_view.verbose > 0 {
            self.fmt_terse(f)?;
            self.fmt_verbose(f)?;
            writeln!(f)
        } else {
            self.fmt_terse(f)
        }
    }
}
impl<'a> MultiView<&'a Device, &'a LspciView> {
    fn fmt_terse(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LspciView {
            basic_view: view,
            vendor_device_subsystem: vds,
            class_code: cc
        } = self.view;
        let device = self.data;
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
        let pciids_class = cc.lookup(class_code.base, None, None);
        let pciids_subclass = cc.lookup(class_code.base, class_code.sub, None);
        match (view.as_numbers, pciids_class, pciids_subclass) {
            (0, _, Some(sub)) =>
                write!(f, " {}:", sub)?,
            (0, Some(base), _) if class_code.base == 0xff =>
                write!(f, " {} [{:02x}{:02x}]:", base, class_code.base, class_code.sub)?,
            (0, Some(base), _) =>
                write!(f, " {}:", base)?,
            (0, _, _) =>
                write!(f, " Class [{:02x}{:02x}]:", class_code.base, class_code.sub)?,
            // Args: -n
            (1, _, _) =>
                write!(f, " {:02x}{:02x}:", class_code.base, class_code.sub)?,
            // Args: -nn+
            (_, _, Some(sub)) => 
                write!(f, " {} [{:02x}{:02x}]:", sub, class_code.base, class_code.sub)?,
            (_, Some(base), _) => 
                write!(f, " {} [{:02x}{:02x}]:", base, class_code.base, class_code.sub)?,
            _ => 
                write!(f, " Class [{:02x}{:02x}]:", class_code.base, class_code.sub)?,
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
            (0, Some(v), _) => write!(f, " {} Device {:04x}", v, device_id)?,
            (0, _, _) => write!(f, " Device {:04x}", device_id)?,
            (1, _, _) => write!(f, " {:04x}:{:04x}", vendor_id, device_id)?,
            (_, Some(v), Some(d)) => write!(f, " {} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id)?,
            (_, Some(v), _) => write!(f, " {} Device [{:04x}:{:04x}]", v, vendor_id, device_id)?,
            _ => write!(f, " Device [{:04x}:{:04x}]", vendor_id, device_id)?,
        }

        // PCI_REVISION_ID
        if revision_id != &0 {
            write!(f, " (rev {:02x})", revision_id)?;
        }

        // PCI_LOOKUP_PROGIF | PCI_LOOKUP_NO_NUMBERS
        let pciids_prog_if = cc.lookup(class_code.base, class_code.sub, class_code.interface);
        if view.verbose > 0 {
            match (class_code.interface, pciids_prog_if) {
                (0, None) => (),
                (_, Some(prog_if)) =>
                    write!(f, " (prog-if {:02x} [{}])", class_code.interface, prog_if)?,
                _ =>
                    write!(f, " (prog-if {:02x})", class_code.interface)?,
            };
        }
        writeln!(f)?;

        if view.verbose > 0 || view.kernel {
            // PCI_FILL_LABEL
            if let Some(label) = &device.label {
                write!(f, "\tDeviceName: {}", label)?;
            }
            // Subdevice
            if let HeaderType::Normal(header::Normal {
                sub_vendor_id: sub_vendor_id @ 0x0001..=0xFFFE,
                sub_device_id,
                ..
            }) = device.header.header_type {
                let pciids_sub_vendor = vds.lookup(sub_vendor_id, None, None);
                let pciids_sub_device = 
                    vds.lookup(*vendor_id, *device_id, (sub_vendor_id, sub_device_id))
                    .or_else(|| vds.lookup(sub_vendor_id, sub_device_id, None));

                // PCI_LOOKUP_SUBSYSTEM | PCI_LOOKUP_VENDOR | PCI_LOOKUP_DEVICE
                match (view.as_numbers, pciids_sub_vendor, pciids_sub_device) {
                    // if (flags & PCI_LOOKUP_NUMERIC)
                    (1, _, _) =>
                        writeln!(f, "\tSubsystem: {:04x}:{:04x}", sub_vendor_id, sub_device_id)?,
                    //  else if (flags & PCI_LOOKUP_MIXED)
                    (2..=usize::MAX, Some(v), Some(d)) => 
                        writeln!(f, "\tSubsystem: {} {} [{:04x}:{:04x}]", v, d, sub_vendor_id, sub_device_id)?,
                    (2..=usize::MAX, None, _) =>
                        writeln!(f, "\tSubsystem: Device [{:04x}:{:04x}]", sub_vendor_id, sub_device_id)?,
                    (2..=usize::MAX, Some(v), _) =>
                        writeln!(f, "\tSubsystem: {} Device [{:04x}:{:04x}]", v, sub_vendor_id, sub_device_id)?,
                    // else
                    (_, Some(v), Some(d)) =>
                        writeln!(f, "\tSubsystem: {} {}", v, d)?,
                    (_, None, _) =>
                        writeln!(f, "\tSubsystem: Device {:04x}", sub_device_id)?,
                    (_, Some(v), None) =>
                        //  In libpci string offset with `device_id + 5`!
                        writeln!(f, "\tSubsystem: {} Device {:04x}", v, sub_device_id)?,
                }
            };
        }
        Ok(())
    }
    fn fmt_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LspciView { basic_view: view, .. } = self.view;
        let Device {
            header: device::Header {
                command,
                status,
                cache_line_size,
                latency_timer,
                header_type,
                bist,
                interrupt_pin,
                class_code,
                ..
            },
            phy_slot,
            numa_node,
            iommu_group,
            ..
        } = self.data;
        if let Some(phy_slot) = &phy_slot {
            writeln!(f, "\tPhysical Slot: {}", phy_slot)?;
        }
        let (min_gnt, max_lat) = {
            let ClassCode { base, sub, .. } = *class_code;
            match header_type {
                HeaderType::Normal(header::Normal { min_grant, max_latency, .. }) => {
                    if base == 0x06 && sub == 0x04 {
                        writeln!(f, "\t!!! Invalid class 0604 for header type 00")?;
                    }
                    (*min_grant as usize * 250, *max_latency as usize * 250)
                },
                HeaderType::Bridge(_) => {
                    if base != 0x06 {
                        writeln!(f, "\t!!! Invalid class {:02x}{:02x} for header type 01", base, sub)?;
                    }
                    (0, 0)
                },
                HeaderType::Cardbus(_) => {
                    if base != 0x06 {
                        writeln!(f, "\t!!! Invalid class {:02x}{:02x} for header type 02", base, sub)?;
                    }
                    (0, 0)
                },
                HeaderType::Reserved(htype) => {
                    writeln!(f, "\t!!! Unknown header type {:02x}", htype)?;
                    return Ok(());
                },
            }
        };

        // TODO: Device tree node

        let irq = self.data.irq();
        if view.verbose > 1 {
            write!(f, "\tControl: {}\n\tStatus: {}\n",
                command.display(()),
                status.display(()),
            )?;
            if command.bus_master {
                write!(f, "\tLatency: {}", latency_timer)?;
                match (min_gnt, max_lat) {
                    (0, 0) => Ok(()),
                    (min, 0) => write!(f, " ({}ns min)", min),
                    (0, max) => write!(f, " ({}ns max)", max),
                    (min, max) => write!(f, " ({}ns min, {}ns max)", min, max),
                }?;
                if cache_line_size != &0 {
                    write!(f, ", Cache Line Size: {} bytes", (*cache_line_size as usize) * 4)?;
                }
                writeln!(f)?;
            }
            match interrupt_pin {
                InterruptPin::Unused if irq != 0 =>
                    writeln!(f, "\tInterrupt: pin ? routed to IRQ {}", irq)?,
                InterruptPin::IntA =>
                    writeln!(f, "\tInterrupt: pin A routed to IRQ {}", irq)?,
                InterruptPin::IntB =>
                    writeln!(f, "\tInterrupt: pin B routed to IRQ {}", irq)?,
                InterruptPin::IntC =>
                    writeln!(f, "\tInterrupt: pin C routed to IRQ {}", irq)?,
                InterruptPin::IntD =>
                    writeln!(f, "\tInterrupt: pin D routed to IRQ {}", irq)?,
                InterruptPin::Reserved(v) => {
                    let u8_as_char = (('A' as u32) + (*v as u32) - 1) & 0xff;
                    if let Some(pin) = char::from_u32(u8_as_char) {
                        writeln!(f, "\tInterrupt: pin {} routed to IRQ {}", pin, irq)?;
                    };
                },
                _ => (),
            };
            if let Some(numa_node) = numa_node {
                writeln!(f, "\tNUMA node: {}", numa_node)?;
            }
            if let Some(iommu_group) = iommu_group {
                writeln!(f, "\tIOMMU group: {}", iommu_group)?;
            }
        } else {
            write!(f, "\tFlags: {}{}{}{}{}{}{} devsel",
                command.bus_master.display(BoolView::Str("bus master, ")),
                command.vga_palette_snoop.display(BoolView::Str("VGA palette snoop, ")),
                command.stepping.display(BoolView::Str("stepping, ")),
                command.fast_back_to_back_enable.display(BoolView::Str("fast Back2Back, ")),
                status.is_66mhz_capable.display(BoolView::Str("66MHz, ")),
                status.user_definable_features.display(BoolView::Str("user-definable features, ")),
                status.devsel_timing.display(()),
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
            writeln!(f)?;
        }
        // BIST
        if bist.is_capable {
            if bist.is_running {
                writeln!(f, "\tBIST is running")?;
            } else {
                writeln!(f, "\tBIST result: {:02x}", bist.completion_code)?;
            }
        }
        match header_type {
            device::HeaderType::Normal(_) => self.fmt_header_normal(f),
            device::HeaderType::Bridge(bridge) => self.fmt_header_bridge(f, bridge),
            device::HeaderType::Cardbus(cardbus) => self.fmt_header_cardbus(f, cardbus),
            _ => Ok(())
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
        let verbose = self.view.basic_view.verbose as u64;
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
        writeln!(f, "\tBus: primary={:02x}, secondary={:02x}, subordinate={:02x}, sec-latency={}",
            primary_bus_number, secondary_bus_number, subordinate_bus_number, secondary_latency_timer
        )?;

        // write!(f, "{:?} ", io_address_range)?;
        // TODO: I/O Base and I/O Limit registers values from /sys/bus/pci/devices/*/resource
        match io_address_range {
            BridgeIoAddressRange::NotImplemented => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, 0, 0xfff, false, verbose)?
            }
            BridgeIoAddressRange::IoAddr16 { base, limit } => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfff, false, verbose)?
            },
            BridgeIoAddressRange::IoAddr32 { base, limit } => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfff, false, verbose)?
            },
            BridgeIoAddressRange::Malformed { base, limit } =>
                writeln!(f, "\t!!! Unknown I/O range types {:x}/{:x}", base, limit)?,
            BridgeIoAddressRange::Reserved { base, limit } =>
                writeln!(f, "\t!!! Unknown I/O range types {:x}/{:x}", base, limit)?,
        }

        // The bottom four bits of both the Memory Base and Memory Limit registers are read-only
        // and return zeros when read.
        if (memory_base & 0xf) != 0 || (memory_limit & 0xf) != 0 {
            writeln!(f, "\t!!! Unknown memory range types {:x}/{:x}", memory_base, memory_limit)?;
        } else {
            let memory_base = ((memory_base & !0xf) as u64) << 16;
            let memory_limit = ((memory_limit & !0xf) as u64) << 16;
            write!(f, "\tMemory behind bridge:")?;
            fmt_range(f, memory_base, memory_limit + 0xfffff, false, verbose)?;
        }
        
        // TODO: Prefetchable Memory Base and Prefetchable Memory Limit values from
        // /sys/bus/pci/devices/*/resource
        match prefetchable_memory {
            BridgePrefetchableMemory::NotImplemented => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, 0, 0xfffff, false, verbose)?
            }
            BridgePrefetchableMemory::MemAddr32 { base, limit } => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfffff, false, verbose)?
            },
            BridgePrefetchableMemory::MemAddr64 { base, limit } => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, *base, *limit + 0xfffff, true, verbose)?
            },
            BridgePrefetchableMemory::Malformed { base, limit } =>
                writeln!(f, "\t!!! Unknown prefetchable memory range types {:x}/{:x}", base, limit)?,
            BridgePrefetchableMemory::Reserved { base, limit } =>
                writeln!(f, "\t!!! Unknown prefetchable memory range types {:x}/{:x}", base, limit)?,
        }
        if verbose > 1 {
            writeln!(f, "\tSecondary status: {}", secondary_status.display(()))?;
        }
        self.fmt_rom(f)?;
        if verbose > 1 {
            writeln!(f,
                "\tBridgeCtl: Parity{} SERR{} NoISA{} VGA{} VGA16{} MAbort{} >Reset{} FastB2B{}",
                bridge_control.parity_error_response_enable.display(BoolView::PlusMinus),
                bridge_control.serr_enable.display(BoolView::PlusMinus),
                bridge_control.isa_enable.display(BoolView::PlusMinus),
                bridge_control.vga_enable.display(BoolView::PlusMinus),
                bridge_control.vga_16_enable.display(BoolView::PlusMinus),
                bridge_control.master_abort_mode.display(BoolView::PlusMinus),
                bridge_control.secondary_bus_reset.display(BoolView::PlusMinus),
                bridge_control.fast_back_to_back_enable.display(BoolView::PlusMinus),
            )?;
            writeln!(f,
                "\t\tPriDiscTmr{} SecDiscTmr{} DiscTmrStat{} DiscTmrSERREn{}",
                bridge_control.primary_discard_timer.display(BoolView::PlusMinus),
                bridge_control.secondary_discard_timer.display(BoolView::PlusMinus),
                bridge_control.discard_timer_status.display(BoolView::PlusMinus),
                bridge_control.discard_timer_serr_enable.display(BoolView::PlusMinus),
            )?;
        }
        self.fmt_capabilities(f)
    }
    // ref to show_htype2(struct device *d);
    fn fmt_header_cardbus(&self, f: &mut fmt::Formatter<'_>, cardbus: &header::Cardbus) -> fmt::Result {
        let verbose = self.view.basic_view.verbose as u64;
        let device::Header {
            command,
            ..
        } = &self.data.header;
        let header::Cardbus {
            secondary_status,
            pci_bus_number,
            cardbus_bus_number,
            subordinate_bus_number,
            cardbus_latency_timer,
            memory_base_address_0,
            memory_limit_address_0,
            memory_base_address_1,
            memory_limit_address_1,
            io_access_address_range_0,
            io_access_address_range_1,
            bridge_control: bctl,
            legacy_mode_base_address,
            reserved,
            ..
        }  = cardbus;
        self.fmt_bases(f)?;
        writeln!(f, "\tBus: primary={:02x}, secondary={:02x}, subordinate={:02x}, sec-latency={}",
            pci_bus_number, cardbus_bus_number, subordinate_bus_number, cardbus_latency_timer
        )?;
        
        let mut fmt_mem_window = |n: usize, base: u32, limit: u32, pf: bool| -> fmt::Result {
            if base <= limit || verbose > 0 {
                writeln!(f, "\tMemory window {}: {:08x}-{:08x}{}{}",
                    n, base, limit.wrapping_add(0xfff),
                    if command.memory_space { "" } else { " [disabled]" },
                    if pf { " (prefetchable)" } else { "" },
                )
            } else {
                Ok(())
            }
        };
        fmt_mem_window(
            0, *memory_base_address_0, *memory_limit_address_0, bctl.memory_0_prefetch_enable
        )?;
        fmt_mem_window(
            1, *memory_base_address_1, *memory_limit_address_1, bctl.memory_1_prefetch_enable
        )?;

        let mut fmt_io_window = |n: usize, range: &IoAccessAddressRange| -> fmt::Result {
            let (base, limit) = match range {
                IoAccessAddressRange::Addr16Bit { base, limit } => {
                    (*base as u32, *limit as u32 + 3) 
                },
                IoAccessAddressRange::Addr32Bit { base, limit } => {
                    (*base, *limit + 3) 
                },
                IoAccessAddressRange::Unknown {
                    io_address_capability, base_lower, base_upper, limit_lower, limit_upper
                } => {
                    // lspci only check first bit of io_address_capability
                    if io_address_capability & 0b1 == 0 {
                        (*base_lower as u32, *limit_lower as u32 + 3) 
                    } else {
                        let base = ((*base_upper as u32) << 16) | (*base_lower as u32);
                        let limit = ((*limit_upper as u32) << 16) | (*limit_lower as u32);
                        (base, limit + 3) 
                    }
                },
            };
            if base <= limit || verbose > 0 {
                writeln!(f, "\tI/O window {}: {:08x}-{:08x}{}",
                    n, base, limit,
                    if command.io_space { "" } else { " [disabled]" },
                )
            } else {
                Ok(())
            }
        };
        fmt_io_window(0, io_access_address_range_0)?;
        fmt_io_window(1, io_access_address_range_1)?;

        if secondary_status.system_error {
            writeln!(f, "\tSecondary status: SERR")?;
        }

        if verbose > 1 {
            writeln!(f,
                "\tBridgeCtl: Parity{} SERR{} ISA{} VGA{} MAbort{} >Reset{} 16bInt{} PostWrite{}",
                bctl.parity_error_response_enable.display(BoolView::PlusMinus),
                bctl.serr_enable.display(BoolView::PlusMinus),
                bctl.isa_enable.display(BoolView::PlusMinus),
                bctl.vga_enable.display(BoolView::PlusMinus),
                bctl.master_abort_mode.display(BoolView::PlusMinus),
                bctl.cardbus_reset.display(BoolView::PlusMinus),
                bctl.ireq_int_enable.display(BoolView::PlusMinus),
                bctl.write_posting_enable.display(BoolView::PlusMinus),
            )?;
        }

        if reserved.is_none() {
            return writeln!(f, "\t<access denied to the rest>");
        }
        if let Some(exca) = legacy_mode_base_address {
            writeln!(f, "\t16-bit legacy interface ports at {:04x}", exca)?;
        }

        self.fmt_capabilities(f)
    }
    fn fmt_bases(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &LspciView {
            basic_view: BasicView { verbose, .. },
            ..
        } = self.view;
        let device = self.data;
        let Command {
            io_space,
            memory_space,
            ..
        } = device.header.command;
        let mut bars = match device.header.header_type {
            HeaderType::Normal(Normal {
                ref base_addresses,
                ..
            }) => base_addresses.0.iter(),
            HeaderType::Bridge(Bridge {
                ref base_addresses, ..
            }) => base_addresses.0.iter(),
            HeaderType::Cardbus(Cardbus {
                ref base_addresses, ..
            }) => base_addresses.0.iter(),
            _ => todo!(),
        }.enumerate();

        const PCI_ADDR_MEM_MASK: u64 = !0xf;
        // const PCI_BASE_ADDRESS_SPACE: u32 = 0x01; /* 0 = memory, 1 = I/O */
        const PCI_BASE_ADDRESS_SPACE_IO: u32 = 0x01;
        // const PCI_BASE_ADDRESS_SPACE_MEMORY: u32 = 0x00;
        const PCI_BASE_ADDRESS_MEM_TYPE_MASK: u32 = 0x06;
        const PCI_BASE_ADDRESS_MEM_TYPE_32: u32 = 0x00; /* 32 bit address */
        const PCI_BASE_ADDRESS_MEM_TYPE_1M: u32 = 0x02; /* Below 1M [obsolete] */
        const PCI_BASE_ADDRESS_MEM_TYPE_64: u32 = 0x04; /* 64 bit address */
        const PCI_BASE_ADDRESS_MEM_PREFETCH: u32 = 0x08; /* prefetchable? */
        const PCI_BASE_ADDRESS_MEM_MASK: u64 = !0x0f;
        const PCI_BASE_ADDRESS_IO_MASK: u64 = !0x03;

        let mut virt = false;
        while let Some((n, bar)) = bars.next() {
            let (pos, len, ioflg) =
                if let Some(re) = device.resource.as_ref().and_then(|r| r.entries.get(n)) {
                    (re.base_addr(), re.size(), re.flags)
                } else {
                    (0, 0, 0)
                };

            let mut flg = *bar;
            let mut hw_upper = 0;
            let mut broken = false;

            if flg == u32::MAX {
                flg = 0;
            }

            // TODO: Complicated base_addr[i] values. Just set from conf space
            let mut pos: u64 = if pos == 0 { flg as u64 } else { pos };

            if pos == 0 && flg == 0 && len == 0 {
                continue;
            }

            if verbose > 1 {
                write!(f, "\tRegion {}: ", n)?;
            } else {
                write!(f, "\t")?;
            }

            // Read address as seen by the hardware
            let hw_lower = if (flg & PCI_BASE_ADDRESS_SPACE_IO) != 0 {
                flg & PCI_BASE_ADDRESS_IO_MASK as u32
            } else {
                if (flg & PCI_BASE_ADDRESS_MEM_TYPE_MASK) == PCI_BASE_ADDRESS_MEM_TYPE_64 {
                    if let Some((_, val)) = bars.next() {
                        hw_upper = *val;
                        // TODO: Complicated base_addr[i] values. Just set from conf space
                        pos |= (hw_upper as u64) << 32;
                    } else {
                        eprintln!("pcilib: {}: Invalid 64-bit address seen for BAR {}.", device.address, n);
                        broken = true;
                    }
                }
                flg & PCI_BASE_ADDRESS_MEM_MASK as u32
            };

            // Detect virtual regions, which are reported by the OS, but unassigned in the device
            if pos != 0 && hw_lower == 0 && hw_upper == 0 && (ioflg & PCI_IORESOURCE_PCI_EA_BEI) == 0 {
                flg = pos as u32;
                virt = true;
            }

            // Print base address
            if (flg & PCI_BASE_ADDRESS_SPACE_IO) != 0 {
                let a = pos & PCI_BASE_ADDRESS_IO_MASK;
                write!(f, "I/O ports at ")?;
                if a != 0 || io_space {
                    write!(f, "{:04x}", a)?;
                } else if hw_lower != 0 {
                    write!(f, "<ignored>")?;
                } else {
                    write!(f, "<unassigned>")?;
                }
                if virt {
                    write!(f, " [virtual]")?;
                } else if !io_space {
                    write!(f, " [disabled]")?;
                }
            } else {
                let t = flg & PCI_BASE_ADDRESS_MEM_TYPE_MASK;
                let a = pos & PCI_ADDR_MEM_MASK;

                write!(f, "Memory at ")?;
                if broken {
                    write!(f, "<broken-64-bit-slot>")?;
                } else if a != 0 {
                    write!(f, "{:08x}", a)?;
                } else if hw_lower != 0 || hw_upper != 0 {
                    write!(f, "<ignored>")?;
                } else {
                    write!(f, "<unassigned>")?;
                }
                let type_ = match t {
                    PCI_BASE_ADDRESS_MEM_TYPE_32 => "32-bit",
                    PCI_BASE_ADDRESS_MEM_TYPE_64 => "64-bit",
                    PCI_BASE_ADDRESS_MEM_TYPE_1M => "low-1M",
                    _ => "type 3",
                };
                let pf = if flg & PCI_BASE_ADDRESS_MEM_PREFETCH != 0 {
                    ""
                } else {
                    "non-"
                };
                write!(f, " ({}, {}prefetchable)", type_, pf)?;
                if virt {
                    write!(f, " [virtual]")?;
                } else if !memory_space {
                    write!(f, " [disabled]")?;
                }
            }

            if ioflg & PCI_IORESOURCE_PCI_EA_BEI != 0 {
                write!(f, " [enhanced]")?;
            }

            fmt_size(f, len)?;
            writeln!(f)?;
        }
        Ok(())
    }



    // fn fmt_machine(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //     write!(f, "TODO: fmt_machine")
    // }

    fn fmt_rom(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const PCI_ROM_ADDRESS_ENABLE: u32 = 0x01;
        const PCI_ROM_ADDRESS_MASK: u32 = !0x7ff;
        const PCI_IORESOURCE_PCI_EA_BEI: u64 = 1 << 5;

        let Device {
            header:
                Header {
                    header_type,
                    command: Command { memory_space, .. },
                    ..
                },
            resource,
            ..
        } = self.data;
        let (_rom, len, ioflg) = if let Some(re) = resource.as_ref().map(|r| r.rom_entry.clone()) {
            (re.base_addr(), re.size(), re.flags)
        } else {
            (0, 0, 0)
        };
        let mut flg: u32 = 0;
        if let Some(expansion_rom) = header_type.expansion_rom() {
            flg = expansion_rom.into();
        }
        let mut virt = false;

        // TODO: Virtual ROM, flags and other
        let rom: u64 = if flg == u32::MAX { 0 } else { flg as u64 };

        if rom == 0 && flg == 0 && len == 0 {
            return Ok(());
        }

        if (rom & PCI_ROM_ADDRESS_MASK as u64) != 0
            && (flg & PCI_ROM_ADDRESS_MASK) == 0
            && (ioflg & PCI_IORESOURCE_PCI_EA_BEI) == 0
        {
            flg = rom as u32;
            virt = true;
        }

        write!(f, "\tExpansion ROM at ")?;
        if (rom & PCI_ROM_ADDRESS_MASK as u64) != 0 {
            write!(f, "{:08x}", rom & PCI_ROM_ADDRESS_MASK as u64)?;
        } else if (flg & PCI_ROM_ADDRESS_MASK) != 0 {
            write!(f, "<ignored>")?;
        } else {
            write!(f, "<unassigned>")?;
        }

        if virt {
            write!(f, " [virtual]")?;
        }

        if (flg & PCI_ROM_ADDRESS_ENABLE) == 0 {
            write!(f, " [disabled]")?;
        } else if !virt && !memory_space {
            write!(f, " [disabled by cmd]")?;
        }

        if (ioflg & PCI_IORESOURCE_PCI_EA_BEI) != 0 {
            write!(f, " [enhanced]")?;
        }

        fmt_size(f, len)?;

        writeln!(f)?;
        Ok(())
    }

    fn fmt_capabilities(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device = self.data;
        let LspciView {
            basic_view: view,
            vendor_device_subsystem: vds,
            ..
        } = self.view;
        if !device.header.status.capabilities_list {
            return Ok(());
        }
        let mut maybe_pci_express = None;
        let caps_view = CapsView {
            view: &view,
            device,
            vds,
        };
        if let Some(caps) = device.capabilities() {
            for cap in caps {
                match cap {
                    Ok(cap) => {
                        write!(f, "{}", cap.display(&caps_view))?;
                        if let CapabilityKind::PciExpress(pci_express) = cap.kind {
                            maybe_pci_express = Some(pci_express);
                        }
                    }
                    Err(err) => write!(f, "{}", err.display(()))?,
                }
            }
        } else {
            writeln!(f, "\tCapabilities: <access denied>")?;
        };
        let ecaps_view = EcapsView {
            view: &view,
            device,
            maybe_pci_express: maybe_pci_express.as_ref(),
        };
        if let Some(ecaps) = device.extended_capabilities() {
            for ecap in ecaps {
                match ecap {
                    Ok(ecap) => write!(f, "{}", ecap.display(&ecaps_view))?,
                    Err(ecap_err) => write!(f, "{}", View((&ecap_err, &ecaps_view)))?,
                }
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
        write!(f, " [size={}{}]", result as u32, suffix[i])
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
        fmt_size(f, limit.wrapping_sub(base) + 1)?;
    }  else {
        write!(f, " [disabled]")?;
    }
    writeln!(f)
}


#[cfg(test)]
mod tests {
    use crate::{
        device::{
            Device,
            ConfigurationSpace,
            address::Address, ResourceEntry, Resource,
        },
        pciids::PciIds,
    };
    // use pcics::header::bar::BaseAddresses;
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref VIEW: LspciView = {
            let (vendor_device_subsystem, class_code) = 
                PciIds::new(include_str!("/usr/share/hwdata/pci.ids").lines())
                    .collect::<(pciids::VendorDeviceSubsystem, pciids::ClassCode)>();
            LspciView {
                basic_view: BasicView::default(),
                vendor_device_subsystem,
                class_code,
            }
        };
        static ref I9DC8: Device = {
            let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/device/8086:9dc8/config"));
            let cs: ConfigurationSpace = data.as_slice().try_into().unwrap();
            let address: Address = "00:1f.3".parse().unwrap();
            let mut device = Device::new(address, cs);
            // Emulate sized base_addresses
            device.resource = Some(Resource {
                entries: [
                    (0x00000000b4418000, 0x00000000b441bfff, 0x0000000000140204),
                    (0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
                    (0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
                    (0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
                    (0x00000000b4100000, 0x00000000b41fffff, 0x0000000000140204),
                    (0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
                ].map(|(start, end, flags)| ResourceEntry { start, end, flags }),
                rom_entry: ResourceEntry { start: 0, end: 0, flags: 0 },
            });
            device.irq = Some(145);
            device
        };
    }

    #[test]
    fn display_device_basic() {
        let view = &*VIEW;
        assert_eq!(
            "00:1f.3 Audio device: Intel Corporation Cannon Point-LP High Definition Audio Controller (rev 30)\n",
            I9DC8.display(view).to_string(),
        );
    }

    mod display_device_as_numbers {
        use super::*;
        macro_rules! display_device_as_numbers {
            ($($id:ident: $sample:expr, $val:expr;)*) => {
                $(
                    #[test]
                    fn $id() {
                        let mut view = VIEW.clone();
                        view.basic_view.as_numbers = $val;
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
                        let mut view = VIEW.clone();
                        view.basic_view.verbose = $val;
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

