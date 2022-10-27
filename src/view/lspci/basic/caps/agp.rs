use core::fmt;

use pcics::capabilities::accelerated_graphics_port::{
    AcceleratedGraphicsPort, DataRateEnabled, DataRateSupport, Identifier,
};

use super::{Flag, Verbose, View};

impl<'a> fmt::Display for Verbose<&'a AcceleratedGraphicsPort> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &AcceleratedGraphicsPort {
            identifier: Identifier { major, minor },
            ref status,
            ref command,
            ..
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "AGP version {:x}.{:x}", major, minor)?;
        if verbose < 2 {
            return Ok(());
        }
        let agp3 = if major >= 3 && status.agp_3_0_mode {
            1
        } else {
            0
        };
        writeln!(f, "\t\tStatus: RQ={} Iso{} ArqSz={} Cal={} SBA{} ITACoh{} GART64{} HTrans{} 64bit{} FW{} AGP3{} Rate={}",
            (status.rq as usize) + 1,
            // [BUG]
            // https://github.com/pciutils/pciutils/blob/864aecdea9c7db626856d8d452f6c784316a878c/lib/header.h#L290
            // should be 17 bit (0x20000) not 16 (0x10000)
            if cfg!(feature = "ls_caps_agp_isoch_support") {
                Flag(status.reserved)
            } else {
                Flag(status.isoch_support)
            },
            status.arqsz,
            Into::<u8>::into(status.cal_cycle),
            Flag(status.sba),
            Flag(status.ita_coh),
            Flag(status.gart64b),
            Flag(status.htrans),
            Flag(status.over4g),
            Flag(status.fw),
            Flag(status.agp_3_0_mode),
            View { data: status.rate, args: agp3 },
        )?;
        writeln!(
            f,
            "\t\tCommand: RQ={} ArqSz={} Cal={} SBA{} AGP{} GART64{} 64bit{} FW{} Rate={}",
            (command.prq as usize) + 1,
            command.parqsz,
            Into::<u8>::into(command.pcal_cycle),
            Flag(command.sba_enable),
            Flag(command.agp_enable),
            Flag(command.gart64b),
            Flag(command.over4g),
            Flag(command.fw_enable),
            View {
                data: command.drate,
                args: agp3
            },
        )?;

        Ok(())
    }
}

impl fmt::Display for View<DataRateSupport, u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_agp_rate(f, (self.data).into(), self.args)
    }
}

impl fmt::Display for View<DataRateEnabled, u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_agp_rate(f, (self.data).into(), self.args)
    }
}

fn format_agp_rate(f: &mut fmt::Formatter<'_>, rate: u8, agp3: u8) -> fmt::Result {
    if rate == 0 {
        write!(f, "<none>")?;
    } else {
        let mut is_not_empty = false;
        for i in 0..=2 {
            if rate & (1 << i) != 0 {
                let comma = if is_not_empty { "," } else { "" };
                is_not_empty = true;
                write!(f, "{}x{}", comma, 1 << ((i as u8) + 2 * agp3))?;
            }
        }
    }
    Ok(())
}
