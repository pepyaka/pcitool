use core::fmt;

use pcics::capabilities::pci_x::{DeviceComplexity, PciX, PciXBridge};

use super::{Flag, Simple, Verbose};

impl<'a> fmt::Display for Verbose<&'a PciX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PciX {
            ref status,
            ref command,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "PCI-X non-bridge device")?;
        if verbose < 2 {
            return Ok(());
        }
        writeln!(
            f,
            "\t\tCommand: DPERE{} ERO{} RBC={} OST={}",
            Flag(command.uncorrectable_data_error_recovery_enable),
            Flag(command.enable_relaxed_ordering),
            command.maximum_memory_read_byte_count.value(),
            command.maximum_outstanding_split_transactions.value(),
        )?;
        writeln!(f, "\t\tStatus: Dev={:02x}:{:02x}.{} 64bit{} 133MHz{} SCD{} USC{} DC={} DMMRBC={} DMOST={} DMCRS={} RSCEM{} 266MHz{} 533MHz{}",
            status.bus_number,
            status.device_number,
            status.function_number,
            Flag(status.device_64_bit),
            Flag(status.pci_x_133_capable),
            Flag(status.slit_completion_discarded),
            Flag(status.unexpected_split_completion),
            Simple(status.device_complexity),
            status.designed_maximum_memory_read_byte_count.value(),
            status.designed_maximum_outstanding_split_transactions.value(),
            status.designed_maximum_cumulative_read_size.adqs(),
            Flag(status.received_split_completion_error_message),
            Flag(status.pci_x_266_capable),
            Flag(status.pci_x_533_capable),
        )?;
        Ok(())
    }
}

impl<'a> fmt::Display for Verbose<&'a PciXBridge> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PciXBridge {
            secondary_status: ref sec,
            bridge_status: ref bri,
            upstream_split_transaction_control: ref up,
            downstream_split_transaction_control: ref down,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "PCI-X bridge device")?;
        if verbose < 2 {
            return Ok(());
        }
        let secondary_bus_mode_and_frequency = match sec.secondary_bus_mode_and_frequency() & 0b111
        {
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
        writeln!(
            f,
            "\t\tSecondary Status: 64bit{} 133MHz{} SCD{} USC{} SCO{} SRD{} Freq={}",
            Flag(sec.device_64_bit),
            Flag(sec.pci_x_133_capable),
            Flag(sec.slit_completion_discarded),
            Flag(sec.unexpected_split_completion),
            Flag(sec.split_completion_overrun),
            Flag(sec.split_request_delayed),
            secondary_bus_mode_and_frequency,
        )?;
        writeln!(
            f,
            "\t\tStatus: Dev={:02x}:{:02x}.{} 64bit{} 133MHz{} SCD{} USC{} SCO{} SRD{}",
            bri.bus_number,
            bri.device_number,
            bri.function_number,
            Flag(bri.device_64_bit),
            Flag(bri.pci_x_133_capable),
            Flag(bri.slit_completion_discarded),
            Flag(bri.unexpected_split_completion),
            Flag(bri.split_completion_overrun),
            Flag(bri.split_request_delayed),
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

impl fmt::Display for Simple<DeviceComplexity> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            DeviceComplexity::Simple => write!(f, "simple"),
            DeviceComplexity::Bridge => write!(f, "bridge"),
        }
    }
}
