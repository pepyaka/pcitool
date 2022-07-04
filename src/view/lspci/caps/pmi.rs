use core::fmt;

use pcics::capabilities::power_management_interface::{PowerManagementInterface, AuxCurrent};

use super::{Flag, View, Verbose};

impl<'a> fmt::Display for View<(&'a PowerManagementInterface, Verbose)> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (PowerManagementInterface {
            capabilities: caps,
            control: ctrl,
            bridge: br,
            ..
        }
        ,Verbose(verbose)) = self.0;
        writeln!(f, "Power Management version {}", caps.version)?;
        if verbose < 2 {
            return Ok(())
        }
        writeln!(f, "\t\tFlags: PMEClk{} DSI{} D1{} D2{} AuxCurrent={} PME(D0{},D1{},D2{},D3hot{},D3cold{})",
            Flag(caps.pme_clock),
            Flag(caps.device_specific_initialization),
            Flag(caps.d1_support),
            Flag(caps.d2_support),
            View(caps.aux_current), 
            Flag(caps.pme_support.d0),
            Flag(caps.pme_support.d1),
            Flag(caps.pme_support.d2),
            Flag(caps.pme_support.d3_hot),
            Flag(caps.pme_support.d3_cold),
        )?;
        writeln!(f, "\t\tStatus: D{} NoSoftRst{} PME-Enable{} DSel={} DScale={:?} PME{}",
            ctrl.power_state as usize,
            Flag(ctrl.no_soft_reset),
            Flag(ctrl.pme_enabled),
            u8::from(ctrl.data_select),
            ctrl.data_scale as usize,
            Flag(ctrl.pme_status),
        )?;
        if br.bpcc_enabled || br.b2_b3 || br.reserved != 0 {
            writeln!(f, "\t\tBridge: PM{} B3{}",
                Flag(br.bpcc_enabled),
                Flag(!br.b2_b3),
            )?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for View<AuxCurrent> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
            AuxCurrent::SelfPowered => "0mA",
            AuxCurrent::MaxCurrent55mA => "55mA",
            AuxCurrent::MaxCurrent100mA => "100mA",
            AuxCurrent::MaxCurrent160mA => "160mA",
            AuxCurrent::MaxCurrent220mA => "220mA",
            AuxCurrent::MaxCurrent270mA => "270mA",
            AuxCurrent::MaxCurrent320mA => "320mA",
            AuxCurrent::MaxCurrent375mA => "375mA",
        };
        write!(f, "{}", s)
    }
}
