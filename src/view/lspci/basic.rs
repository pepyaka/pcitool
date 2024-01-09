use core::fmt;

use pcics::{
    capabilities::CapabilityKind,
    header::{
        self, Bridge, BridgeIoAddressRange, BridgePrefetchableMemory, Cardbus, ClassCode, Command,
        Header, HeaderType, InterruptPin, IoAccessAddressRange, Normal,
    },
};

use crate::{access::Access, device::Device, names};

mod caps;
mod ecaps;
mod hdr;

const PCI_IORESOURCE_PCI_EA_BEI: u64 = 1 << 5;

// Bool view wrapper
pub struct Flag(pub bool);

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.0 { "+" } else { "-" })
    }
}

// Show message on true or empty string
pub struct FlagMsg(pub bool, &'static str);

impl fmt::Display for FlagMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.0 { self.1 } else { "" })
    }
}

// Verbose view wrapper
struct Verbose<T> {
    data: T,
    verbose: usize,
}

/// Simple view wrapper
struct Simple<T>(pub T);

/// Wrapper around any type for adding [fmt::Display] trait
pub struct View<T, V> {
    pub data: T,
    pub args: V,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewArgs<'a> {
    pub verbose: usize,
    pub kernel: bool,
    pub always_domain_number: bool,
    pub as_numbers: usize,
    pub vds: &'a names::VendorDeviceSubsystem,
    pub cc: &'a names::ClassCode,
    pub access: &'a Access,
}

impl<'a> fmt::Display for View<Device, &'a ViewArgs<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &ViewArgs {
            verbose, kernel, ..
        } = self.args;
        if verbose > 0 {
            self.fmt_terse(f)?;
            self.fmt_verbose(f)?;
            self.fmt_kernel(f)?;
            writeln!(f)?;
            Ok(())
        } else {
            self.fmt_terse(f)?;
            if kernel {
                self.fmt_kernel(f)?;
            }
            Ok(())
        }
    }
}

impl<'a> View<Device, &'a ViewArgs<'a>> {
    fn fmt_terse(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Device {
            ref address,
            ref label,
            header:
                Header {
                    ref header_type,
                    vendor_id,
                    device_id,
                    revision_id,
                    ref class_code,
                    ..
                },
            ..
        } = self.data;
        let &ViewArgs {
            verbose,
            as_numbers,
            vds,
            cc,
            always_domain_number,
            kernel,
            ..
        } = self.args;
        // Device address
        if always_domain_number {
            write!(f, "{:}", address)?;
        } else {
            write!(f, "{:#}", address)?;
        }

        // PCI_LOOKUP_CLASS
        let class_name = fmt_class_name(
            as_numbers,
            class_code.base,
            class_code.sub,
            cc.lookup(class_code.base, None, None).as_deref(),
            cc.lookup(class_code.base, class_code.sub, None).as_deref(),
            128,
        );

        // PCI_LOOKUP_VENDOR | PCI_LOOKUP_DEVICE
        let device_name = fmt_device_name(
            as_numbers,
            vendor_id,
            device_id,
            vds.lookup(vendor_id, None, None).as_deref(),
            vds.lookup(vendor_id, device_id, None).as_deref(),
            128,
        );
        write!(f, " {}: {}", class_name, device_name)?;

        // PCI_REVISION_ID
        if revision_id != 0 {
            write!(f, " (rev {:02x})", revision_id)?;
        }

        // PCI_LOOKUP_PROGIF | PCI_LOOKUP_NO_NUMBERS
        if verbose > 0 {
            let prg_if_name = cc
                .lookup(class_code.base, class_code.sub, class_code.interface)
                /* IDE controllers have complex prog-if semantics */
                .or_else(|| {
                    if class_code.base == 0x01
                        && class_code.sub == 0x01
                        && class_code.interface & 0x70 == 0
                    {
                        Some(format!(
                            "{}{}{}{}{}",
                            if class_code.interface & 0x80 != 0 {
                                " Master"
                            } else {
                                ""
                            },
                            if class_code.interface & 0x08 != 0 {
                                " SecP"
                            } else {
                                ""
                            },
                            if class_code.interface & 0x04 != 0 {
                                " SecO"
                            } else {
                                ""
                            },
                            if class_code.interface & 0x02 != 0 {
                                " PriP"
                            } else {
                                ""
                            },
                            if class_code.interface & 0x01 != 0 {
                                " PriO"
                            } else {
                                ""
                            },
                        ))
                    } else {
                        None
                    }
                });
            if let Some(x) = prg_if_name {
                write!(
                    f,
                    " (prog-if {:02x} [{}])",
                    class_code.interface,
                    x.trim_start()
                )?;
            } else if class_code.interface > 0 {
                write!(f, " (prog-if {:02x})", class_code.interface)?;
            }
        }
        writeln!(f)?;

        if verbose > 0 || kernel {
            // PCI_FILL_LABEL
            if let Some(label) = label {
                write!(f, "\tDeviceName: {}", label)?;
            }
            // Subdevice
            if let &HeaderType::Normal(header::Normal {
                sub_vendor_id: sub_vendor_id @ 0x0001..=0xFFFE,
                sub_device_id,
                ..
            }) = header_type
            {
                let sub_vendor_name = vds.lookup(sub_vendor_id, None, None);
                // Per-device lookup
                let mut sub_device_name = if vendor_id > 0 && device_id > 0 {
                    vds.lookup(vendor_id, device_id, (sub_vendor_id, sub_device_id))
                } else {
                    None
                };
                // neither pci.ids nor hwdb has generic subsystems
                // // Generic lookup
                // if sub_device_name.is_none() {
                //     sub_device_name = vds.lookup(None, None, (sub_vendor_id, sub_device_id))
                // }
                // Check for subsystem == device
                if sub_device_name.is_none()
                    && vendor_id == sub_vendor_id
                    && device_id == sub_device_id
                {
                    sub_device_name = vds.lookup(vendor_id, device_id, None)
                };

                // PCI_LOOKUP_SUBSYSTEM | PCI_LOOKUP_VENDOR | PCI_LOOKUP_DEVICE
                let subsys_name = fmt_device_name(
                    as_numbers,
                    sub_vendor_id,
                    sub_device_id,
                    sub_vendor_name.as_deref(),
                    sub_device_name.as_deref(),
                    256,
                );
                writeln!(f, "\tSubsystem: {}", subsys_name)?;
            };
        }
        Ok(())
    }
    fn fmt_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ref device @ Device {
            header:
                Header {
                    ref command,
                    ref status,
                    cache_line_size,
                    latency_timer,
                    ref header_type,
                    ref bist,
                    interrupt_pin,
                    ref class_code,
                    ..
                },
            numa_node,
            ref phy_slot,
            ref iommu_group,
            ..
        } = self.data;
        let &ViewArgs { verbose, .. } = self.args;
        if let Some(phy_slot) = phy_slot {
            writeln!(f, "\tPhysical Slot: {}", phy_slot)?;
        }
        let (min_gnt, max_lat) = {
            let &ClassCode { base, sub, .. } = class_code;
            match header_type {
                &HeaderType::Normal(header::Normal {
                    min_grant,
                    max_latency,
                    ..
                }) => {
                    if base == 0x06 && sub == 0x04 {
                        writeln!(f, "\t!!! Invalid class 0604 for header type 00")?;
                    }
                    (min_grant as usize * 250, max_latency as usize * 250)
                }
                HeaderType::Bridge(_) => {
                    if base != 0x06 {
                        writeln!(
                            f,
                            "\t!!! Invalid class {:02x}{:02x} for header type 01",
                            base, sub
                        )?;
                    }
                    (0, 0)
                }
                HeaderType::Cardbus(_) => {
                    if base != 0x06 {
                        writeln!(
                            f,
                            "\t!!! Invalid class {:02x}{:02x} for header type 02",
                            base, sub
                        )?;
                    }
                    (0, 0)
                }
                HeaderType::Reserved(htype) => {
                    writeln!(f, "\t!!! Unknown header type {:02x}", htype)?;
                    return Ok(());
                }
            }
        };

        // TODO: Device tree node

        let irq = device.irq();
        if verbose > 1 {
            write!(
                f,
                "\tControl: {}\n\tStatus: {}\n",
                Simple(command.clone()),
                Simple(status.clone()),
            )?;
            if command.bus_master {
                write!(f, "\tLatency: {}", latency_timer)?;
                match (min_gnt, max_lat) {
                    (0, 0) => Ok(()),
                    (min, 0) => write!(f, " ({}ns min)", min),
                    (0, max) => write!(f, " ({}ns max)", max),
                    (min, max) => write!(f, " ({}ns min, {}ns max)", min, max),
                }?;
                if cache_line_size != 0 {
                    write!(
                        f,
                        ", Cache Line Size: {} bytes",
                        (cache_line_size as usize) * 4
                    )?;
                }
                writeln!(f)?;
            }
            match interrupt_pin {
                InterruptPin::Unused if irq != 0 => {
                    writeln!(f, "\tInterrupt: pin ? routed to IRQ {}", irq)?
                }
                InterruptPin::IntA => writeln!(f, "\tInterrupt: pin A routed to IRQ {}", irq)?,
                InterruptPin::IntB => writeln!(f, "\tInterrupt: pin B routed to IRQ {}", irq)?,
                InterruptPin::IntC => writeln!(f, "\tInterrupt: pin C routed to IRQ {}", irq)?,
                InterruptPin::IntD => writeln!(f, "\tInterrupt: pin D routed to IRQ {}", irq)?,
                InterruptPin::Reserved(v) => {
                    let u8_as_char = (('A' as u32) + (v as u32) - 1) & 0xff;
                    if let Some(pin) = char::from_u32(u8_as_char) {
                        writeln!(f, "\tInterrupt: pin {} routed to IRQ {}", pin, irq)?;
                    };
                }
                _ => (),
            };
            if let Some(numa_node) = numa_node {
                writeln!(f, "\tNUMA node: {}", numa_node)?;
            }
            if let Some(iommu_group) = iommu_group {
                writeln!(f, "\tIOMMU group: {}", iommu_group)?;
            }
        } else {
            write!(
                f,
                "\tFlags: {}{}{}{}{}{}{} devsel",
                FlagMsg(command.bus_master, "bus master, "),
                FlagMsg(command.vga_palette_snoop, "VGA palette snoop, "),
                FlagMsg(command.stepping, "stepping, "),
                FlagMsg(command.fast_back_to_back_enable, "fast Back2Back, "),
                FlagMsg(status.is_66mhz_capable, "66MHz, "),
                FlagMsg(status.user_definable_features, "user-definable features, "),
                Simple(status.devsel_timing),
            )?;
            if command.bus_master {
                write!(f, ", latency {}", latency_timer)?;
            }
            if irq != 0 {
                #[cfg(target_arch = "sparc64")]
                write!(f, ", IRQ {:08x}", irq)?;
                #[cfg(not(target_arch = "sparc64"))]
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
            HeaderType::Normal(_) => self.fmt_header_normal(f),
            HeaderType::Bridge(bridge) => self.fmt_header_bridge(f, bridge),
            HeaderType::Cardbus(cardbus) => self.fmt_header_cardbus(f, cardbus),
            _ => Ok(()),
        }
    }
    fn fmt_kernel(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Device {
            ref driver_in_use,
            #[cfg(feature = "pciutils_make_opt_libkmod")]
            ref kernel_modules,
            ..
        } = self.data;
        if let Some(driver_in_use) = driver_in_use {
            writeln!(f, "\tKernel driver in use: {}", driver_in_use)?;
        }
        #[cfg(feature = "pciutils_make_opt_libkmod")]
        if let Some(kernel_modules) = kernel_modules {
            if !kernel_modules.is_empty() {
                writeln!(f, "\tKernel modules: {}", kernel_modules.join(", "))?;
            }
        }
        Ok(())
    }
    // ref to show_htype0(struct device *d);
    fn fmt_header_normal(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_bases(f)?;
        self.fmt_rom(f)?;
        self.fmt_capabilities(f)
    }
    // ref to show_htype1(struct device *d);
    fn fmt_header_bridge(
        &self,
        f: &mut fmt::Formatter<'_>,
        bridge: &header::Bridge,
    ) -> fmt::Result {
        let &ViewArgs { verbose, .. } = self.args;
        let verbose = verbose as u64;
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
        } = bridge;
        self.fmt_bases(f)?;
        writeln!(
            f,
            "\tBus: primary={:02x}, secondary={:02x}, subordinate={:02x}, sec-latency={}",
            primary_bus_number,
            secondary_bus_number,
            subordinate_bus_number,
            secondary_latency_timer
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
            }
            BridgeIoAddressRange::IoAddr32 { base, limit } => {
                write!(f, "\tI/O behind bridge:")?;
                fmt_range(f, *base as u64, *limit as u64 + 0xfff, false, verbose)?
            }
            BridgeIoAddressRange::Malformed { base, limit } => {
                writeln!(f, "\t!!! Unknown I/O range types {:x}/{:x}", base, limit)?
            }
            BridgeIoAddressRange::Reserved { base, limit } => {
                writeln!(f, "\t!!! Unknown I/O range types {:x}/{:x}", base, limit)?
            }
        }

        // The bottom four bits of both the Memory Base and Memory Limit registers are read-only
        // and return zeros when read.
        if (memory_base & 0xf) != 0 || (memory_limit & 0xf) != 0 {
            writeln!(
                f,
                "\t!!! Unknown memory range types {:x}/{:x}",
                memory_base, memory_limit
            )?;
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
            }
            BridgePrefetchableMemory::MemAddr64 { base, limit } => {
                write!(f, "\tPrefetchable memory behind bridge:")?;
                fmt_range(f, *base, *limit + 0xfffff, true, verbose)?
            }
            BridgePrefetchableMemory::Malformed { base, limit } => writeln!(
                f,
                "\t!!! Unknown prefetchable memory range types {:x}/{:x}",
                base, limit
            )?,
            BridgePrefetchableMemory::Reserved { base, limit } => writeln!(
                f,
                "\t!!! Unknown prefetchable memory range types {:x}/{:x}",
                base, limit
            )?,
        }
        if verbose > 1 {
            writeln!(
                f,
                "\tSecondary status: {}",
                Simple(secondary_status.clone())
            )?;
        }
        self.fmt_rom(f)?;
        if verbose > 1 {
            writeln!(
                f,
                "\tBridgeCtl: Parity{} SERR{} NoISA{} VGA{} VGA16{} MAbort{} >Reset{} FastB2B{}",
                Flag(bridge_control.parity_error_response_enable),
                Flag(bridge_control.serr_enable),
                Flag(bridge_control.isa_enable),
                Flag(bridge_control.vga_enable),
                Flag(bridge_control.vga_16_enable),
                Flag(bridge_control.master_abort_mode),
                Flag(bridge_control.secondary_bus_reset),
                Flag(bridge_control.fast_back_to_back_enable),
            )?;
            writeln!(
                f,
                "\t\tPriDiscTmr{} SecDiscTmr{} DiscTmrStat{} DiscTmrSERREn{}",
                Flag(bridge_control.primary_discard_timer),
                Flag(bridge_control.secondary_discard_timer),
                Flag(bridge_control.discard_timer_status),
                Flag(bridge_control.discard_timer_serr_enable),
            )?;
        }
        self.fmt_capabilities(f)
    }
    // ref to show_htype2(struct device *d);
    fn fmt_header_cardbus(
        &self,
        f: &mut fmt::Formatter<'_>,
        cardbus: &header::Cardbus,
    ) -> fmt::Result {
        let Device {
            header: Header { ref command, .. },
            ..
        } = self.data;
        let &ViewArgs { verbose, .. } = self.args;
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
        } = cardbus;
        self.fmt_bases(f)?;
        writeln!(
            f,
            "\tBus: primary={:02x}, secondary={:02x}, subordinate={:02x}, sec-latency={}",
            pci_bus_number, cardbus_bus_number, subordinate_bus_number, cardbus_latency_timer
        )?;

        let mut fmt_mem_window = |n: usize, base: u32, limit: u32, pf: bool| -> fmt::Result {
            if base <= limit || verbose > 0 {
                writeln!(
                    f,
                    "\tMemory window {}: {:08x}-{:08x}{}{}",
                    n,
                    base,
                    limit.wrapping_add(0xfff),
                    if command.memory_space {
                        ""
                    } else {
                        " [disabled]"
                    },
                    if pf { " (prefetchable)" } else { "" },
                )
            } else {
                Ok(())
            }
        };
        fmt_mem_window(
            0,
            *memory_base_address_0,
            *memory_limit_address_0,
            bctl.memory_0_prefetch_enable,
        )?;
        fmt_mem_window(
            1,
            *memory_base_address_1,
            *memory_limit_address_1,
            bctl.memory_1_prefetch_enable,
        )?;

        let mut fmt_io_window = |n: usize, range: &IoAccessAddressRange| -> fmt::Result {
            let (base, limit) = match range {
                IoAccessAddressRange::Addr16Bit { base, limit } => {
                    (*base as u32, *limit as u32 + 3)
                }
                IoAccessAddressRange::Addr32Bit { base, limit } => (*base, *limit + 3),
                IoAccessAddressRange::Unknown {
                    io_address_capability,
                    base_lower,
                    base_upper,
                    limit_lower,
                    limit_upper,
                } => {
                    // lspci only check first bit of io_address_capability
                    if io_address_capability & 0b1 == 0 {
                        (*base_lower as u32, *limit_lower as u32 + 3)
                    } else {
                        let base = ((*base_upper as u32) << 16) | (*base_lower as u32);
                        let limit = ((*limit_upper as u32) << 16) | (*limit_lower as u32);
                        (base, limit + 3)
                    }
                }
            };
            if base <= limit || verbose > 0 {
                writeln!(
                    f,
                    "\tI/O window {}: {:08x}-{:08x}{}",
                    n,
                    base,
                    limit,
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
            writeln!(
                f,
                "\tBridgeCtl: Parity{} SERR{} ISA{} VGA{} MAbort{} >Reset{} 16bInt{} PostWrite{}",
                Flag(bctl.parity_error_response_enable),
                Flag(bctl.serr_enable),
                Flag(bctl.isa_enable),
                Flag(bctl.vga_enable),
                Flag(bctl.master_abort_mode),
                Flag(bctl.cardbus_reset),
                Flag(bctl.ireq_int_enable),
                Flag(bctl.write_posting_enable),
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
        let Device {
            ref address,
            ref resource,
            header:
                Header {
                    ref header_type,
                    command:
                        Command {
                            io_space,
                            memory_space,
                            ..
                        },
                    ..
                },
            ..
        } = self.data;
        let &ViewArgs { verbose, .. } = self.args;
        let mut bars_data = [0u32; 6];
        let mut bars = match header_type {
            HeaderType::Normal(Normal { base_addresses, .. }) => {
                bars_data[..6].clone_from_slice(base_addresses.orig().as_slice());
                &bars_data[..6]
            }
            HeaderType::Bridge(Bridge { base_addresses, .. }) => {
                bars_data[..2].clone_from_slice(base_addresses.orig().as_slice());
                &bars_data[..2]
            }
            HeaderType::Cardbus(Cardbus { base_addresses, .. }) => {
                bars_data[..1].clone_from_slice(base_addresses.orig().as_slice());
                &bars_data[..1]
            }
            _ => [].as_slice(),
        }
        .iter()
        .peekable()
        .enumerate();

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
                if let Some(re) = resource.as_ref().and_then(|r| r.entries.get(n)) {
                    (re.base_addr(), re.size(), re.flags)
                } else {
                    (0, 0, 0)
                };

            let mut flg = *bar;
            let hw_lower;
            let mut hw_upper = 0;
            let mut broken = false;

            if flg == u32::MAX {
                flg = 0;
            }

            if pos == 0 && flg == 0 && len == 0 {
                continue;
            }

            if verbose > 1 {
                write!(f, "\tRegion {}: ", n)?;
            } else {
                write!(f, "\t")?;
            }

            // Read address as seen by the hardware
            if (flg & PCI_BASE_ADDRESS_SPACE_IO) != 0 {
                hw_lower = flg & PCI_BASE_ADDRESS_IO_MASK as u32;
            } else {
                hw_lower = flg & PCI_BASE_ADDRESS_MEM_MASK as u32;
                if (flg & PCI_BASE_ADDRESS_MEM_TYPE_MASK) == PCI_BASE_ADDRESS_MEM_TYPE_64 {
                    if let Some((_, val)) = bars.next() {
                        hw_upper = *val;
                    } else {
                        eprintln!(
                            "pcilib: {}: Invalid 64-bit address seen for BAR {}.",
                            address, n
                        );
                        broken = true;
                    }
                }
            };

            // generic.c by default fill base_addr[] with values from configuration space
            // we will emulate this
            let pos = if resource.is_some() {
                pos
            } else {
                flg as u64 | (hw_upper as u64) << 32
            };

            // Detect virtual regions, which are reported by the OS, but unassigned in the device
            if pos != 0
                && hw_lower == 0
                && hw_upper == 0
                && (ioflg & PCI_IORESOURCE_PCI_EA_BEI) == 0
            {
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
                    ref header_type,
                    command: Command { memory_space, .. },
                    ..
                },
            ref resource,
            ..
        } = self.data;
        let mut flg: u32 = header_type.expansion_rom().map(Into::into).unwrap_or(0);
        let (rom, len, ioflg) = if let Some(re) = resource.as_ref().map(|r| r.rom_entry) {
            (re.base_addr(), re.size(), re.flags)
        } else {
            let rom: u64 = if flg == u32::MAX { 0 } else { flg as u64 };
            (rom, 0, 0)
        };
        let mut virt = false;

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
        let device = &self.data;
        let &ViewArgs {
            vds,
            verbose,
            as_numbers,
            access,
            ..
        } = self.args;
        if !device.header.status.capabilities_list {
            return Ok(());
        }
        let maybe_pci_express = if let Some(caps) = device.capabilities() {
            let mut maybe_pci_express = None;
            let args = &caps::ViewArgs {
                verbose,
                as_numbers,
                device,
                vds,
                access,
            };
            for cap in caps {
                match cap {
                    Ok(data) => {
                        if let CapabilityKind::PciExpress(ref pci_express) = data.kind {
                            maybe_pci_express = Some(pci_express.clone());
                        }
                        write!(f, "{}", View { data, args })?;
                    }
                    Err(data) => write!(f, "{}", Simple(data))?,
                }
            }
            maybe_pci_express
        } else {
            writeln!(f, "\tCapabilities: <access denied>")?;
            None
        };

        if let Some(ecaps) = device.extended_capabilities() {
            let args = &ecaps::ViewArgs {
                verbose,
                device,
                maybe_pci_express: maybe_pci_express.as_ref(),
            };
            for ecap in ecaps {
                match ecap {
                    Ok(data) => write!(f, "{}", View { data, args }),
                    Err(data) => write!(f, "{}", Verbose { data, verbose }),
                }?;
            }
        }
        Ok(())
    }
}

fn fmt_size(f: &mut fmt::Formatter<'_>, x: u64) -> fmt::Result {
    let suffix = ["", "K", "M", "G", "T"];
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

fn fmt_range(
    f: &mut fmt::Formatter<'_>,
    base: u64,
    limit: u64,
    is_64bit: bool,
    verbose: u64,
) -> fmt::Result {
    if base <= limit || verbose > 2 {
        if is_64bit {
            write!(f, " {:016x}-{:016x}", base, limit)?;
        } else {
            write!(f, " {:08x}-{:08x}", base, limit)?;
        }
    }
    if base <= limit {
        fmt_size(f, limit.wrapping_sub(base) + 1)?;
    } else {
        write!(f, " [disabled]")?;
    }
    writeln!(f)
}

// Wrap string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    // sizeof(buf[128]) minus '\0'
    let max_len = max_len - 1;
    let len = s.len();
    if len >= max_len && len >= 4 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn fmt_class_name(
    as_numbers: usize,
    base_id: u8,
    sub_id: u8,
    base_name: Option<&str>,
    sub_name: Option<&str>,
    max_len: usize,
) -> String {
    let maybe_long_str = match (as_numbers, base_name, sub_name) {
        (0, _, Some(sub)) => sub.to_string(),
        (0, Some(base), _) => {
            format!("{} [{:02x}{:02x}]", base, base_id, sub_id)
        }
        (0, _, _) => format!("Class {:02x}{:02x}", base_id, sub_id),
        // Args: -n
        (1, _, _) => format!("{:02x}{:02x}", base_id, sub_id),
        // Args: -nn+
        (_, _, Some(sub)) => format!("{} [{:02x}{:02x}]", sub, base_id, sub_id),
        (_, Some(base), _) => {
            format!("{} [{:02x}{:02x}]", base, base_id, sub_id)
        }
        _ => format!("Class [{:02x}{:02x}]", base_id, sub_id),
    };
    truncate(&maybe_long_str, max_len)
}

fn fmt_device_name(
    as_numbers: usize,
    vendor_id: u16,
    device_id: u16,
    vendor_name: Option<&str>,
    device_name: Option<&str>,
    max_len: usize,
) -> String {
    let maybe_long_str = match (as_numbers, vendor_name, device_name) {
        (0, Some(v), Some(d)) => format!("{} {}", v, d),
        (0, Some(v), _) => format!("{} Device {:04x}", v, device_id),
        (0, _, _) => format!("Device {:04x}:{:04x}", vendor_id, device_id),
        (1, _, _) => format!("{:04x}:{:04x}", vendor_id, device_id),
        (_, Some(v), Some(d)) => {
            format!("{} {} [{:04x}:{:04x}]", v, d, vendor_id, device_id)
        }
        (_, Some(v), _) => format!("{} Device [{:04x}:{:04x}]", v, vendor_id, device_id),
        _ => format!("Device [{:04x}:{:04x}]", vendor_id, device_id),
    };
    truncate(&maybe_long_str, max_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{address::Address, ConfigurationSpace, Device, Resource, ResourceEntry};
    use crate::names::Names;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_str_eq;

    lazy_static! {
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
        let names = Names::init().unwrap_or_default();
        let vds = &names.vendor_device_subsystem();
        let cc = &names.class_code();
        let args = &ViewArgs {
            verbose: 0,
            kernel: false,
            always_domain_number: false,
            as_numbers: 0,
            vds,
            cc,
            access: &Default::default(),
        };
        assert_str_eq!(
            "00:1f.3 Audio device: Intel Corporation Cannon Point-LP High Definition Audio Controller (rev 30)\n",
            View { data: I9DC8.clone(), args }.to_string(),
        );
    }

    mod display_device_as_numbers {
        use super::*;
        macro_rules! display_device_as_numbers {
            ($($id:ident: $sample:expr, $val:expr;)*) => {
                $(
                    #[test]
                    fn $id() {
                        let names = Names::init().unwrap_or_default();
                        let vds = &names.vendor_device_subsystem();
                        let cc = &names.class_code();
                        let args = &ViewArgs {
                            verbose: 0,
                            kernel: false,
                            always_domain_number: false,
                            as_numbers: $val,
                            vds,
                            cc,
                            access: &Default::default(),
                        };
                        let result = View { data: I9DC8.clone(), args }.to_string();
                        assert_str_eq!($sample, result);
                    }
                )*
            };
        }
        display_device_as_numbers! {
            eq_1: "00:1f.3 0403: 8086:9dc8 (rev 30)\n", 1;
        }
    }
    mod display_device_verbose {
        use super::*;
        macro_rules! display_device_verbose {
            ($($id:ident: $out:expr, $val:expr;)*) => {
                $(
                    #[test]
                    fn $id() {
                        let names = Names::init().unwrap_or_default();
                        let vds = &names.vendor_device_subsystem();
                        let cc = &names.class_code();
                        let args = &ViewArgs {
                            verbose: $val,
                            kernel: false,
                            always_domain_number: false,
                            as_numbers: 0,
                            vds,
                            cc,
                            access: &Default::default(),
                        };
                        let result = View { data: I9DC8.clone(), args }.to_string();
                        let sample =
                            include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
                            "/tests/data/device/", $out)
                            );
                        assert_str_eq!(sample, result);
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

    #[test]
    fn long_names_ellipsis() {
        let data = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:2f68/config"
        ));
        let cs: ConfigurationSpace = data.as_slice().try_into().unwrap();
        let address: Address = "7f:16.0".parse().unwrap();
        let device = Device::new(address, cs);
        let names = Names::init().unwrap_or_default();
        let vds = &names.vendor_device_subsystem();
        let cc = &names.class_code();
        let args = &ViewArgs {
            verbose: 0,
            kernel: false,
            always_domain_number: false,
            as_numbers: 2,
            vds,
            cc,
            access: &Default::default(),
        };
        let result = View { data: device, args }.to_string();
        let sample = "7f:16.0 System peripheral [0880]: Intel Corporation Xeon E7 v3/Xeon E5 v3/Core i7 Integrated Memory Controller 1 Target Address, Thermal & RAS Registers [8086... (rev 02)\n";
        assert_str_eq!(sample, result);
    }

    #[test]
    fn caps_pointer_equal_zero() {
        let data = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:2f83/config"
        ));
        let cs: ConfigurationSpace = data.as_slice().try_into().unwrap();
        let address: Address = "7f:08.3".parse().unwrap();
        let device = Device::new(address, cs);
        let names = Names::init_pciids(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pci.ids"))
            .unwrap_or_default();
        let vds = &names.vendor_device_subsystem();
        let cc = &names.class_code();
        let args = &ViewArgs {
            verbose: 2,
            kernel: false,
            always_domain_number: false,
            as_numbers: 0,
            vds,
            cc,
            access: &Default::default(),
        };
        let result = View { data: device, args }.to_string();
        let sample = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/device/8086:2f83/out.vv.txt"
        ));
        assert_str_eq!(sample, result);
    }
}
