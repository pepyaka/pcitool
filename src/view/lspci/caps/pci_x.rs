use core::fmt;

use pcics::capabilities::pci_x::{DeviceComplexity, PciX, PciXBridge};

use crate::view::{BoolView, DisplayMultiView, MultiView, Verbose};

impl<'a> DisplayMultiView<Verbose> for PciX {}
impl<'a> fmt::Display for MultiView<&'a PciX, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        let PciX {
            status, command, ..
        } = self.data;
        writeln!(f, "PCI-X non-bridge device")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tCommand: DPERE{} ERO{} RBC={} OST={}",
            command
                .uncorrectable_data_error_recovery_enable
                .display(BoolView::PlusMinus),
            command.enable_relaxed_ordering.display(BoolView::PlusMinus),
            command.maximum_memory_read_byte_count.value(),
            command.maximum_outstanding_split_transactions.value(),
        )?;
        writeln!(f, "\t\tStatus: Dev={:02x}:{:02x}.{} 64bit{} 133MHz{} SCD{} USC{} DC={} DMMRBC={} DMOST={} DMCRS={} RSCEM{} 266MHz{} 533MHz{}",
            status.bus_number,
            status.device_number,
            status.function_number,
            status.device_64_bit.display(BoolView::PlusMinus),
            status.pci_x_133_capable.display(BoolView::PlusMinus),
            status.slit_completion_discarded.display(BoolView::PlusMinus),
            status.unexpected_split_completion.display(BoolView::PlusMinus),
            status.device_complexity.display(()),
            status.designed_maximum_memory_read_byte_count.value(),
            status.designed_maximum_outstanding_split_transactions.value(),
            status.designed_maximum_cumulative_read_size.adqs(),
            status.received_split_completion_error_message.display(BoolView::PlusMinus),
            status.pci_x_266_capable.display(BoolView::PlusMinus),
            status.pci_x_533_capable.display(BoolView::PlusMinus),
        )?;
        Ok(())
    }
}

impl<'a> DisplayMultiView<Verbose> for PciXBridge {}
impl<'a> fmt::Display for MultiView<&'a PciXBridge, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        let PciXBridge {
            secondary_status: sec,
            bridge_status: bri,
            upstream_split_transaction_control: up,
            downstream_split_transaction_control: down,
            ..
        } = self.data;
        writeln!(f, "PCI-X bridge device")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tSecondary Status: 64bit{} 133MHz{} SCD{} USC{} SCO{} SRD{} Freq={}",
            sec.device_64_bit.display(BoolView::PlusMinus),
            sec.pci_x_133_capable.display(BoolView::PlusMinus),
            sec.slit_completion_discarded.display(BoolView::PlusMinus),
            sec.unexpected_split_completion.display(BoolView::PlusMinus),
            sec.split_completion_overrun.display(BoolView::PlusMinus),
            sec.split_request_delayed.display(BoolView::PlusMinus),
            SecondaryBusModeAndFrequency(sec.secondary_bus_mode_and_frequency()).display(()),
        )?;
        writeln!(
            f,
            "\t\tStatus: Dev={:02x}:{:02x}.{} 64bit{} 133MHz{} SCD{} USC{} SCO{} SRD{}",
            bri.bus_number,
            bri.device_number,
            bri.function_number,
            bri.device_64_bit.display(BoolView::PlusMinus),
            bri.pci_x_133_capable.display(BoolView::PlusMinus),
            bri.slit_completion_discarded.display(BoolView::PlusMinus),
            bri.unexpected_split_completion.display(BoolView::PlusMinus),
            bri.split_completion_overrun.display(BoolView::PlusMinus),
            bri.split_request_delayed.display(BoolView::PlusMinus),
        )?;
        writeln!(
            f,
            "\t\tUpstream: Capacity={} CommitmentLimit={}",
            up.split_transaction_capacity, up.split_transaction_commitment_limit,
        )?;
        writeln!(
            f,
            "\t\tDownstream: Capacity={} CommitmentLimit={}",
            down.split_transaction_capacity, down.split_transaction_commitment_limit,
        )?;
        Ok(())
    }
}

impl<'a> DisplayMultiView<()> for DeviceComplexity {}
impl<'a> fmt::Display for MultiView<&'a DeviceComplexity, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            DeviceComplexity::Simple => write!(f, "simple"),
            DeviceComplexity::Bridge => write!(f, "bridge"),
        }
    }
}

struct SecondaryBusModeAndFrequency(u8);

impl<'a> DisplayMultiView<()> for SecondaryBusModeAndFrequency {}
impl<'a> fmt::Display for MultiView<&'a SecondaryBusModeAndFrequency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data.0 & 0b111 {
            0 => "conv",
            1 => "66MHz",
            2 => "100MHz",
            3 => "133MHz",
            4 => "?4",
            5 => "?5",
            6 => "?6",
            7 => "?7",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}
