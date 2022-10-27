use core::fmt;

use pcics::capabilities::power_management_interface::{AuxCurrent, PowerManagementInterface};

use super::Flag;

pub(super) struct View<'a> {
    pub(super) pmi: &'a PowerManagementInterface,
    pub(super) raw_data: &'a [u8],
    pub(super) verbose: usize,
}

impl<'a> fmt::Display for View<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &View {
            pmi:
                PowerManagementInterface {
                    capabilities: caps,
                    control: ctrl,
                    bridge: br,
                    ..
                },
            verbose,
            raw_data,
        } = self;
        writeln!(f, "Power Management version {}", caps.version)?;
        if verbose < 2 {
            return Ok(());
        }
        let aux_current = match caps.aux_current {
            AuxCurrent::SelfPowered => "0mA",
            AuxCurrent::MaxCurrent55mA => "55mA",
            AuxCurrent::MaxCurrent100mA => "100mA",
            AuxCurrent::MaxCurrent160mA => "160mA",
            AuxCurrent::MaxCurrent220mA => "220mA",
            AuxCurrent::MaxCurrent270mA => "270mA",
            AuxCurrent::MaxCurrent320mA => "320mA",
            AuxCurrent::MaxCurrent375mA => "375mA",
        };
        writeln!(f, "\t\tFlags: PMEClk{} DSI{} D1{} D2{} AuxCurrent={} PME(D0{},D1{},D2{},D3hot{},D3cold{})",
            Flag(caps.pme_clock),
            Flag(caps.device_specific_initialization),
            Flag(caps.d1_support),
            Flag(caps.d2_support),
            aux_current,
            Flag(caps.pme_support.d0),
            Flag(caps.pme_support.d1),
            Flag(caps.pme_support.d2),
            Flag(caps.pme_support.d3_hot),
            Flag(caps.pme_support.d3_cold),
        )?;
        writeln!(
            f,
            "\t\tStatus: D{} NoSoftRst{} PME-Enable{} DSel={} DScale={:?} PME{}",
            ctrl.power_state as usize,
            Flag(ctrl.no_soft_reset),
            Flag(ctrl.pme_enabled),
            u8::from(ctrl.data_select),
            ctrl.data_scale as usize,
            Flag(ctrl.pme_status),
        )?;
        if br.bpcc_enabled || br.b2_b3 || br.reserved != 0 {
            let (pm, b3) = if cfg!(feature = "ls_caps_pm_bridge") {
                let raw_ctrl_lo = raw_data.get(0x04).cloned().unwrap_or_default();
                let b2_b3 = (raw_ctrl_lo & 0x40) != 0;
                let bpcc_enabled = (raw_ctrl_lo & 0x80) != 0;
                (bpcc_enabled, !b2_b3)
            } else {
                (br.bpcc_enabled, !br.b2_b3)
            };
            writeln!(f, "\t\tBridge: PM{} B3{}", Flag(pm), Flag(b3))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn power_management_interface() {
        let data = [0x02, 0x7e, 0x00, 0x00, 0x40, 0x00];
        let pmi: PowerManagementInterface = data.as_slice().try_into().unwrap();

        let v1_result = View {
            pmi: &pmi,
            verbose: 1,
            raw_data: &data,
        };
        let v1_sample = "\
            Power Management version 2\n\
        ";
        assert_eq!(v1_sample, v1_result.to_string(), "-v");

        let v2_result = View {
            pmi: &pmi,
            verbose: 2,
            raw_data: &data,
        };
        let v2_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3-\n\
        ";
        assert_eq!(v2_sample, v2_result.to_string(), "-vv");

        let v3_result = View {
            pmi: &pmi,
            verbose: 3,
            raw_data: &data,
        };
        let v3_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3-\n\
        ";
        assert_eq!(v3_sample, v3_result.to_string(), "-vvv");
    }
}
