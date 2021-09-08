//! PCI configuration space is the underlying way that the Conventional PCI, PCI-X and PCI Express
//! perform auto configuration of the cards inserted into their bus.
//!

use std::cmp::Ordering;

use bitfield_layout::{BitFieldLayout, Layout, };

pub mod view;

pub mod address;
pub use address::Address;

pub mod header;
pub use header::{Header, HeaderCommon, HeaderTypeNormal, BaseAddress, bar::BaseAddressSized, BaseAddresses};

#[derive(Debug, Clone, PartialEq, Eq,)] 
pub struct Device {
    pub address: Address,
    pub header: Header,
    /// Device name as exported by BIOS
    pub label: Option<String>,
    /// Physical slot
    pub phy_slot: Option<String>,
    /// NUMA node
    pub numa_node: Option<u16>,
    /// IOMMU group
    pub iommu_group: Option<String>,
    /// Real IRQ
    pub irq: Option<u8>,
}


impl Device {
    pub fn new(address: Address, header: Header) -> Self {
        Self { 
            address, 
            header, 
            label: None, 
            phy_slot: None, 
            numa_node: None, 
            iommu_group: None,
            irq: None,
        }
    }
    pub fn irq(&self) -> u8 {
        if let Some(irq) = self.irq {
            irq
        } else {
            match &self.header {
                Header::Normal(l) => l.interrupt_line,
                Header::Bridge(l) => l.interrupt_line,
                Header::Cardbus(l) => l.interrupt_line,
            }
        }
    }
}
impl PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.address.partial_cmp(&other.address)
    }
}
impl Ord for Device {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

//impl Device {
//    pub fn control<'a, T: AccessMethod>(&mut self, access: &'a mut T, command: Command) -> Result<Command, AccessError> {
//        access.control(command)
//    }
//    pub fn status<'a, T: AccessMethod>(&mut self, access: &'a mut T, reset: Status) -> Result<Status, AccessError> {
//        access.status(reset)
//    }
//}



#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;
    use super::header::*;


    #[test]
    fn text_header_type_bridge() {
        // PCI bridge [0604]: Renesas Technology Corp. SH7758 PCIe Switch [PS] [1912:001d] (prog-if 00 [Normal decode])
        // Control: I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx-
        // Status: Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
        // Latency: 0
        // BIST result: 00
        // Bus: primary=04, secondary=05, subordinate=08, sec-latency=0
        // Memory behind bridge: 92000000-929fffff
        // Prefetchable memory behind bridge: 0000000091000000-0000000091ffffff
        // Secondary status: 66MHz- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- <SERR- <PERR-
        // BridgeCtl: Parity+ SERR+ NoISA- VGA+ MAbort- >Reset- FastB2B-
        //         PriDiscTmr- SecDiscTmr- DiscTmrStat- DiscTmrSERREn-
        // Capabilities: <access denied>
        // Kernel driver in use: pcieport
        let data = [
            0x12, 0x19, 0x1d, 0x00, 0x07, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0x06, 0x00, 0x00, 0x01, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x05, 0x08, 0x00, 0xf1, 0x01, 0x00, 0x00,
            0x00, 0x92, 0x90, 0x92, 0x01, 0x91, 0xf1, 0x91, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x1b, 0x00,
        ];
        let decoded: HeaderTypeBridge = bincode::deserialize(&data[..]).unwrap();
        println!("{:#04X?}", &decoded);

        assert_eq!(
            ("Bridge", Some("PCI bridge"), Some("Normal decode")),
            decoded.common.class_code.meaning(),
            "PCI bridge [0604]"
        );
        assert_eq!(0x1912, decoded.common.vendor_id, "Renesas Technology Corp.");
        assert_eq!(0x001d, decoded.common.device_id, "SH7758 PCIe Switch [PS]");
        assert_eq!(0x00, decoded.common.class_code.interface, "prog-if 00 [Normal decode]");

        let command_result = decoded.common.command.flags()
            .take(11)
            .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(
            "I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx-",
            command_result,
            "Control"
        );

        let status_result = {
            let v = decoded.common.status.flags()
                .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
                .collect::<Vec<String>>();
            format!("{} {}", &v[4..].join(" "), v[3])
        };
        assert_eq!(
            "Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=medium- DEVSEL=slow- >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-",
            status_result,
            "Status"
        );

        assert_eq!(0x00, decoded.common.latency_timer, "Latency");

        let bist_result =
            if decoded.common.bist.is_capable {
                if decoded.common.bist.is_running {
                    Err("BIST is running")
                } else {
                    Ok(decoded.common.bist.completion_code)
                }
            } else {
                Err("Not capable")
            };
        assert_eq!(Ok(0x00), bist_result, "BIST");

        assert_eq!(
            (0x04, 0x05, 0x08, 0x00),
            (decoded.primary_bus_number, decoded.secondary_bus_number,
             decoded.subordinate_bus_number, decoded.secondary_latency_timer),
            "Bus: primary=04, secondary=05, subordinate=08, sec-latency=0"
        );
        assert_eq!(
            (0x92000000u32, 0x929fffffu32),
            ((decoded.memory_base as u32) << 16, ((decoded.memory_limit as u32) << 16) + 0xfffff),
            "Memory behind bridge: 92000000-929fffff"
        );
        assert_eq!(
            (0x91000000u64, 0x91ffffffu64),
            ((decoded.prefetchable_memory_base as u64 & !0x0f) << 16, ((decoded.prefetchable_memory_limit as u64 & !0x0f) << 16) + 0xfffff),
            "Prefetchable memory behind bridge: 0000000091000000-0000000091ffffff" 
        );

        let secondary_status_result = {
            let v = decoded.common.status.flags()
                .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
                .collect::<Vec<String>>();
            format!("{} {}", &v[4..].join(" "), v[3])
        };
        assert_eq!(
            "Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=medium- DEVSEL=slow- >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-",
            secondary_status_result,
            "Secondary status"
        );
            
        let bridge_control_result = decoded.bridge_control.flags()
            .take(12)
            .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
            .collect::<Vec<String>>()
            .join(" ");
        assert_eq!(
            "Parity+ SERR+ NoISA- VGA+ VGA16+ MAbort- >Reset- FastB2B- PriDiscTmr- SecDiscTmr- DiscTmrStat- DiscTmrSERREn-",
            bridge_control_result,
            "Bridge control"
        );
    }

}
