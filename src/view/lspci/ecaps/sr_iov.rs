use core::fmt;

use pcics::{
    extended_capabilities::single_root_io_virtualization::SingleRootIoVirtualization,
    header::{
        BaseAddressType,
        BaseAddresses
    }
};

use crate::view::{DisplayMultiViewBasic, MultiView, Verbose, BoolView};

impl DisplayMultiViewBasic<Verbose> for SingleRootIoVirtualization {}
impl<'a> fmt::Display for MultiView<&'a SingleRootIoVirtualization, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Single Root I/O Virtualization (SR-IOV)")?;
        if verbose < 2 {
            return Ok(())
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
        writeln!(f,
            "\t\tIOVCap:\tMigration{}, Interrupt Message Number: {:03x}",
            sriov_capabilities.vf_migration_capable.display(BoolView::PlusMinus),
            sriov_capabilities.vf_migration_interrupt_message_number,
        )?;
        writeln!(f,
            "\t\tIOVCtl:\tEnable{} Migration{} Interrupt{} MSE{} ARIHierarchy{}",
            sriov_control.vf_enable.display(BoolView::PlusMinus),
            sriov_control.vf_migration_enable.display(BoolView::PlusMinus),
            sriov_control.vf_migration_interrupt_enable.display(BoolView::PlusMinus),
            sriov_control.vf_mse.display(BoolView::PlusMinus),
            sriov_control.ari_capable_hierarchy.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tIOVSta:\tMigration{}",
            sriov_status.vf_migration_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tInitial VFs: {}, Total VFs: {}, Number of VFs: {}, Function Dependency Link: {:02x}",
            initial_vfs, total_vfs, num_vfs, function_dependency_link,
        )?;
        writeln!(f,
            "\t\tVF offset: {}, stride: {}, Device ID: {:04x}",
            first_vf_offset, vf_stride, vf_device_id,
        )?;
        writeln!(f,
            "\t\tSupported Page Size: {:08x}, System Page Size: {:08x}",
            page_sizes.supported, page_sizes.system,
        )?;
        
        let pf = |prefetchable: bool| if prefetchable { "" } else { "non-" };
        for ba in BaseAddresses::from(base_addresses.clone()) {
            write!(f, "\t\tRegion {}: Memory at ", ba.region)?;
            match ba.base_address_type {
                BaseAddressType::MemorySpace32 { base_address, prefetchable, .. } => {
                    writeln!(f, "{:08x} (32-bit, {}prefetchable)", base_address, pf(prefetchable))?;
                },
                BaseAddressType::MemorySpace64 { base_address, prefetchable, .. } => {
                    writeln!(f, "{:016x} (64-bit, {}prefetchable)", base_address, pf(prefetchable))?;
                },
                _ => (),
            };
        }

        writeln!(f,
            "\t\tVF Migration: offset: {:08x}, BIR: {:x}",
            vf_migration_state_array_offset & 0xfffffff8, 
            vf_migration_state_array_offset & 7, 
        )?;
        Ok(())
    }
}
