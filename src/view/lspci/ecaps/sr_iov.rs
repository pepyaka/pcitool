use core::fmt;

use pcics::extended_capabilities::single_root_io_virtualization::SingleRootIoVirtualization;

use crate::view::{BoolView, DisplayMultiView, MultiView, Verbose};

impl DisplayMultiView<Verbose> for SingleRootIoVirtualization {}
impl<'a> fmt::Display for MultiView<&'a SingleRootIoVirtualization, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Single Root I/O Virtualization (SR-IOV)")?;
        if verbose < 2 {
            return Ok(());
        }
        let SingleRootIoVirtualization {
            sriov_capabilities,
            sriov_control,
            sriov_status,
            initial_vfs,
            total_vfs,
            num_vfs,
            function_dependency_link,
            first_vf_offset,
            vf_stride,
            vf_device_id,
            page_sizes,
            base_addresses,
            vf_migration_state_array_offset,
        } = self.data;
        writeln!(
            f,
            "\t\tIOVCap:\tMigration{}, Interrupt Message Number: {:03x}",
            sriov_capabilities
                .vf_migration_capable
                .display(BoolView::PlusMinus),
            sriov_capabilities.vf_migration_interrupt_message_number,
        )?;
        writeln!(
            f,
            "\t\tIOVCtl:\tEnable{} Migration{} Interrupt{} MSE{} ARIHierarchy{}",
            sriov_control.vf_enable.display(BoolView::PlusMinus),
            sriov_control
                .vf_migration_enable
                .display(BoolView::PlusMinus),
            sriov_control
                .vf_migration_interrupt_enable
                .display(BoolView::PlusMinus),
            sriov_control.vf_mse.display(BoolView::PlusMinus),
            sriov_control
                .ari_capable_hierarchy
                .display(BoolView::PlusMinus),
        )?;
        writeln!(
            f,
            "\t\tIOVSta:\tMigration{}",
            sriov_status
                .vf_migration_status
                .display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tInitial VFs: {}, Total VFs: {}, Number of VFs: {}, Function Dependency Link: {:02x}",
            initial_vfs, total_vfs, num_vfs, function_dependency_link,
        )?;
        writeln!(
            f,
            "\t\tVF offset: {}, stride: {}, Device ID: {:04x}",
            first_vf_offset, vf_stride, vf_device_id,
        )?;
        writeln!(
            f,
            "\t\tSupported Page Size: {:08x}, System Page Size: {:08x}",
            page_sizes.supported, page_sizes.system,
        )?;

        let mut vf_bars = base_addresses.0.iter().enumerate();
        while let Some((n, bar)) = vf_bars.next() {
            const PCI_ADDR_MEM_MASK: u32 = !0xf;
            const PCI_BASE_ADDRESS_MEM_TYPE_MASK: u32 = 0x06;
            const PCI_BASE_ADDRESS_MEM_TYPE_32: u32 = 0x00;
            const PCI_BASE_ADDRESS_MEM_TYPE_64: u32 = 0x04;
            const PCI_BASE_ADDRESS_MEM_PREFETCH: u32 = 0x08;
            if bar == &0 || bar == &u32::MAX {
                continue
            }
            let addr = bar & PCI_ADDR_MEM_MASK;
            let type_ = bar & PCI_BASE_ADDRESS_MEM_TYPE_MASK;
            write!(f, "\t\tRegion {}: Memory at ", n)?;
            if type_ == PCI_BASE_ADDRESS_MEM_TYPE_64 {
                // lspci: on last bar shows out of bounds u32 value
                if n == 5 {
                    write!(f, "{:08x}", vf_migration_state_array_offset)?;
                } else if let Some((_, addr_h)) = vf_bars.next() {
                    write!(f, "{:08x}", addr_h)?;
                }
            }
            writeln!(
                f, "{:08x} ({}-bit, {}prefetchable)",
                addr,
                if type_ == PCI_BASE_ADDRESS_MEM_TYPE_32 { "32"} else {"64"},
                if bar & PCI_BASE_ADDRESS_MEM_PREFETCH != 0 { ""} else {"non-"}
            )?;
        }

        writeln!(
            f,
            "\t\tVF Migration: offset: {:08x}, BIR: {:x}",
            vf_migration_state_array_offset & 0xfffffff8,
            vf_migration_state_array_offset & 7,
        )?;
        Ok(())
    }
}
