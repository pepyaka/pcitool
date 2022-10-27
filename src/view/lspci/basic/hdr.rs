use core::fmt;

use pcics::header::{Command, DevselTiming, Status};

use super::{Flag, Simple};

impl fmt::Display for Simple<Command> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "I/O{} Mem{} BusMaster{} SpecCycle{} MemWINV{} VGASnoop{} ParErr{} Stepping{} SERR{} FastB2B{} DisINTx{}",
            Flag(self.0.io_space),
            Flag(self.0.memory_space),
            Flag(self.0.bus_master),
            Flag(self.0.special_cycles),
            Flag(self.0.memory_write_and_invalidate_enable),
            Flag(self.0.vga_palette_snoop),
            Flag(self.0.parity_error_response),
            Flag(self.0.stepping),
            Flag(self.0.serr_enable),
            Flag(self.0.fast_back_to_back_enable),
            Flag(self.0.interrupt_disable),
        )
    }
}

impl fmt::Display for Simple<Status<'P'>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cap{} 66MHz{} UDF{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} \
            >SERR{} <PERR{} INTx{}",
            Flag(self.0.capabilities_list),
            Flag(self.0.is_66mhz_capable),
            Flag(self.0.user_definable_features),
            Flag(self.0.fast_back_to_back_capable),
            Flag(self.0.master_data_parity_error),
            Simple(self.0.devsel_timing),
            Flag(self.0.signaled_target_abort),
            Flag(self.0.received_target_abort),
            Flag(self.0.received_master_abort),
            Flag(self.0.system_error),
            Flag(self.0.detected_parity_error),
            Flag(self.0.interrupt_status),
        )
    }
}

impl fmt::Display for Simple<Status<'B'>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "66MHz{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} <SERR{} <PERR{}",
            Flag(self.0.is_66mhz_capable),
            Flag(self.0.fast_back_to_back_capable),
            Flag(self.0.master_data_parity_error),
            Simple(self.0.devsel_timing),
            Flag(self.0.signaled_target_abort),
            Flag(self.0.received_target_abort),
            Flag(self.0.received_master_abort),
            Flag(self.0.system_error),
            Flag(self.0.detected_parity_error),
        )
    }
}

impl fmt::Display for Simple<Status<'C'>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.system_error {
            write!(f, "SERR")
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Simple<DevselTiming> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            DevselTiming::Fast => write!(f, "fast"),
            DevselTiming::Medium => write!(f, "medium"),
            DevselTiming::Slow => write!(f, "slow"),
            DevselTiming::Undefined => write!(f, "??"),
        }
    }
}
