use core::fmt;

use pcics::capabilities::accelerated_graphics_port::{
    AcceleratedGraphicsPort, DataRateEnabled, DataRateSupport, Identifier,
};

use crate::view::{BoolView, DisplayMultiView, MultiView, Verbose};

impl<'a> DisplayMultiView<Verbose> for AcceleratedGraphicsPort {}
impl<'a> fmt::Display for MultiView<&'a AcceleratedGraphicsPort, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        let AcceleratedGraphicsPort {
            identifier: Identifier { major, minor },
            status,
            command,
            ..
        } = self.data;
        writeln!(f, "AGP version {:x}.{:x}", major, minor)?;
        if verbose < 2 {
            return Ok(());
        }
        let agp3 = if *major >= 3 && status.agp_3_0_mode {
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
                status.reserved.display(BoolView::PlusMinus)
            } else {
                status.isoch_support.display(BoolView::PlusMinus)
            },
            status.arqsz,
            Into::<u8>::into(status.cal_cycle),
            status.sba.display(BoolView::PlusMinus),
            status.ita_coh.display(BoolView::PlusMinus),
            status.gart64b.display(BoolView::PlusMinus),
            status.htrans.display(BoolView::PlusMinus),
            status.over4g.display(BoolView::PlusMinus),
            status.fw.display(BoolView::PlusMinus),
            status.agp_3_0_mode.display(BoolView::PlusMinus),
            status.rate.display(Agp3(agp3)),
        )?;
        writeln!(
            f,
            "\t\tCommand: RQ={} ArqSz={} Cal={} SBA{} AGP{} GART64{} 64bit{} FW{} Rate={}",
            (command.prq as usize) + 1,
            command.parqsz,
            Into::<u8>::into(command.pcal_cycle),
            command.sba_enable.display(BoolView::PlusMinus),
            command.agp_enable.display(BoolView::PlusMinus),
            command.gart64b.display(BoolView::PlusMinus),
            command.over4g.display(BoolView::PlusMinus),
            command.fw_enable.display(BoolView::PlusMinus),
            command.drate.display(Agp3(agp3)),
        )?;

        Ok(())
    }
}

struct Agp3(u8);

impl DisplayMultiView<Agp3> for DataRateSupport {}
impl<'a> fmt::Display for MultiView<&'a DataRateSupport, Agp3> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_agp_rate(f, (*self.data).into(), self.view.0)
    }
}

impl DisplayMultiView<Agp3> for DataRateEnabled {}
impl<'a> fmt::Display for MultiView<&'a DataRateEnabled, Agp3> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_agp_rate(f, (*self.data).into(), self.view.0)
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
