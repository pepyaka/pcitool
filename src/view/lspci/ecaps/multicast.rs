use std::fmt;

use pcics::{
    capabilities::pci_express::DeviceType,
    extended_capabilities::multicast::{
        McBaseAddress, McOverlayBar, Multicast, MulticastCapability, MulticastControl,
    },
};

use crate::view::lspci::Flag;

pub struct MulticastView<'a> {
    pub data: &'a Multicast,
    pub verbose: usize,
    pub maybe_device_type: Option<&'a DeviceType>,
}
impl<'a> fmt::Display for MulticastView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MulticastView {
            data:
                Multicast {
                    multicast_capability:
                        MulticastCapability {
                            mc_max_group,
                            mc_window_size_requested,
                            mc_ecrc_regeneration_supported,
                        },
                    multicast_control:
                        MulticastControl {
                            mc_num_group,
                            mc_enable,
                        },
                    mc_base_address:
                        McBaseAddress {
                            mc_index_position,
                            mc_base_address,
                        },
                    mc_receive,
                    mc_block_all,
                    mc_block_untranslated,
                    mc_overlay_bar,
                },
            verbose,
            maybe_device_type,
        } = self;
        writeln!(f, "Multicast")?;
        if verbose < &2 {
            return Ok(());
        }
        write!(f, "\t\tMcastCap: MaxGroups {:}", mc_max_group + 1)?;
        let is_ep_or_rcip = matches!(
            maybe_device_type,
            Some(DeviceType::Endpoint { .. } | DeviceType::RootComplexIntegratedEndpoint)
        );
        if is_ep_or_rcip {
            write!(
                f,
                ", WindowSz {} ({} bytes)",
                mc_window_size_requested,
                1i32.wrapping_shl(*mc_window_size_requested as u32),
            )?;
        }
        if matches!(
            maybe_device_type,
            Some(
                DeviceType::RootPort { .. }
                    | DeviceType::UpstreamPort { .. }
                    | DeviceType::DownstreamPort { .. }
            )
        ) {
            writeln!(f, ", ECRCRegen{}", Flag(*mc_ecrc_regeneration_supported))?;
        }
        writeln!(
            f,
            "\t\tMcastCtl: NumGroups {}, Enable{}",
            mc_num_group + 1,
            Flag(*mc_enable)
        )?;
        writeln!(
            f,
            "\t\tMcastBAR: IndexPos {}, BaseAddr {:016x}",
            mc_index_position, mc_base_address
        )?;
        writeln!(f, "\t\tMcastReceiveVec:      {:016x}", mc_receive)?;
        writeln!(f, "\t\tMcastBlockAllVec:     {:016x}", mc_block_all)?;
        writeln!(
            f,
            "\t\tMcastBlockUntransVec: {:016x}",
            mc_block_untranslated
        )?;

        if is_ep_or_rcip {
            return Ok(());
        }
        if let Some(McOverlayBar {
            mc_overlay_size,
            mc_overlay_bar,
        }) = mc_overlay_bar
        {
            write!(f, "\t\tMcastOverlayBAR: OverlaySize {} ", mc_overlay_size)?;
            if mc_overlay_size >= &6 {
                write!(f, "({} bytes)", 1i32.wrapping_shl(*mc_overlay_size as u32))?;
            } else {
                write!(f, "(disabled)")?;
            }
            writeln!(f, ", BaseAddr {:016}", mc_overlay_bar)?;
        }
        Ok(())
    }
}
