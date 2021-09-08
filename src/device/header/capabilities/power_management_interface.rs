//! PCI Power Management Interface
//!
//! This capability structure provides a standard interface to control power management features in
//! a PCI device. It is fully documented in the PCI Power Management Interface Specification.

use core::cell::Cell;
use std::fmt::{self, Display, Formatter};

use modular_bitfield::prelude::*;
use displaydoc::Display as DisplayDoc;

use crate::{DisplayView, View, CheckBox};


/// Raw bit struct for [PowerManagementInterface]
#[bitfield(bits = 48)]
#[derive(Debug, PartialEq, Eq,)] 
pub struct PowerManagementRegisterBlock {
    // capability_id: u8,
    // next_item_ptr: u8,
    caps_version: B3,
    caps_pme_clock: bool,
    caps_reserved: bool,
    caps_device_specific_initialization: bool,
    #[bits = 3]
    aux_current: AuxCurrent,
    caps_d1_support: bool,
    caps_d2_support: bool,
    pme_support_d0: bool,
    pme_support_d1: bool,
    pme_support_d2: bool,
    pme_support_d3_hot: bool,
    pme_support_d3_cold: bool,
    #[bits = 2]
    power_state: PowerState,
    ctrl_reserved: B6,
    ctrl_pme_enabled: bool,
    #[bits = 4]
    data_select: DataSelect,
    #[bits = 2]
    data_scale: DataScale,
    ctrl_pme_status: bool,
    br_reserved: B6,
    br_b2_b3: bool,
    br_bpcc_enabled: bool,
    data_value: u8,
}

#[derive(Debug, PartialEq, Eq,)] 
pub struct PowerManagementInterface {
    pub capabilities: Capabilities,
    pub control: Control,
    pub bridge: Bridge,
    pub data: Option<Data>,
    _view: Cell<View>,
}

/// Provides information on the capabilities of the function related to power management
#[derive(Debug, PartialEq, Eq)]
pub struct Capabilities {
    /// Default value of 0b10 indicates that this function complies with Revision 1.1 of the PCI
    /// Power Management Interface Specification.
    pub version: u8,
    /// Indicates that the function relies on the presence of the PCI clock for PME# operation.
    pub pme_clock: bool,
    /// Reserved read-only.
    pub reserved: bool,
    /// Device Specific Initialization (DSI) bit indicates whether special initialization of this
    /// function is required before the generic class device driver is able to use it.
    pub device_specific_initialization: bool,
    pub aux_current: AuxCurrent,
    /// Supports the D1 Power Management State.
    pub d1_support: bool,
    /// Supports the D2 Power Management State.
    pub d2_support: bool,
    pub pme_support: PmeSupport,
}

/// This 3 bit field reports the 3.3Vaux auxiliary current requirements for the PCI function.
/// he [Data] Register takes precedence over this field for 3.3Vaux current and value must be 0.
#[derive(DisplayDoc, BitfieldSpecifier, Debug, PartialEq, Eq)]
#[bits = 3]
pub enum AuxCurrent {
    /// 0mA
    SelfPowered,
    /// 55mA
    MaxCurrent55mA,
    /// 100mA
    MaxCurrent100mA,
    /// 160mA
    MaxCurrent160mA,
    /// 220mA
    MaxCurrent220mA,
    /// 270mA
    MaxCurrent270mA,
    /// 320mA
    MaxCurrent320mA,
    /// 375mA
    MaxCurrent375mA,
}

/// Indicates the power states in which the function may assert PME#.
#[derive(Debug, PartialEq, Eq)]
pub struct PmeSupport {
    /// PME# can be asserted from D0
    pub d0: bool,
    /// PME# can be asserted from D1
    pub d1: bool,
    /// PME# can be asserted from D2
    pub d2: bool,
    /// PME# can be asserted from D3 *hot*
    pub d3_hot: bool,
    /// PME# can be asserted from D3 *cold*
    pub d3_cold: bool,
}

/// Used to manage the PCI function’s power management state as well as to enable/monitor PMEs.
#[derive(Debug, PartialEq, Eq)]
pub struct Control {
    pub power_state: PowerState,
    /// Reserved bits 07:02
    pub reserved: u8,
    /// PCI_PM_CTRL_NO_SOFT_RESET
    pub no_soft_reset: bool,
    /// Enables the function to assert PME#.
    pub pme_enabled: bool,
    /// This bit is set when the function would normally assert the PME# signal independent of the
    /// state of the [pme_enabled] bit.
    pub pme_status: bool,
}

/// Current power state.
#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum PowerState {
    D0,
    D1,
    D2,
    D3Hot,
}

/// PCI bridge specific functionality and is required for all PCI-toPCI bridges
#[derive(Debug, PartialEq, Eq)]
pub struct Bridge {
    /// Value at reset 0b000000
    pub reserved: u8,
    /// B2_B3# (b2/B3 support for D3hot)
    ///
    /// This field determines the action that is to occur as a direct result of programming the
    /// function to D3Hot
    pub b2_b3: bool,
    /// BPCC_En (Bus Power/Clock Control Enable)
    ///
    /// Indicates that the bus power/clock control mechanism is enabled
    pub bpcc_enabled: bool,
}

/// Register that provides a mechanism for the function to report state dependent operating data
/// such as power consumed or heat dissipation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data {
    pub value: u8,
    pub select: DataSelect,
    pub scale: DataScale,
}

/// Used to select which data is to be reported through the [Data] register and [DataScale].
#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 4]
pub enum DataSelect {
    /// D0 Power Consumed
    PowerConsumedD0,
    /// D1 Power Consumed
    PowerConsumedD1,
    /// D2 Power Consumed
    PowerConsumedD2,
    /// D3 Power Consumed
    PowerConsumedD3,
    /// D0 Power Dissipated
    PowerDissipatedD0,
    /// D1 Power Dissipated
    PowerDissipatedD1,
    /// D2 Power Dissipated
    PowerDissipatedD2,
    /// D3 Power Dissipated
    PowerDissipatedD3,
    /// Common logic power consumption
    CommonLogic,
    /// TBD
    Reserved,
}

/// Scaling factor indicated to arrive at the value for the desired measurement.
#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum DataScale {
    Unknown,
    /// 0.1x
    Tenth,
    /// 0.01x
    Hundredth,
    /// 0.001x
    Thousandth,
}


impl<'a> DisplayView<'a> for PowerManagementInterface {
    type View = &'a Self;
    fn display(&'a self, view: View) -> Self::View {
        self._view.set(view);
        self
    }
}
impl From<[u8; 6]> for PowerManagementInterface {
    fn from(data: [u8; 6]) -> Self {
        let pmrb = PowerManagementRegisterBlock::from_bytes(data);
        Self {
            capabilities: Capabilities {
                version: pmrb.caps_version(),
                pme_clock: pmrb.caps_pme_clock(),
                reserved: pmrb.caps_reserved(),
                device_specific_initialization: pmrb.caps_device_specific_initialization(),
                aux_current: pmrb.aux_current(),
                d1_support: pmrb.caps_d1_support(),
                d2_support: pmrb.caps_d2_support(),
                pme_support: PmeSupport {
                    d0: pmrb.pme_support_d0(),
                    d1: pmrb.pme_support_d1(),
                    d2: pmrb.pme_support_d2(),
                    d3_hot: pmrb.pme_support_d3_hot(),
                    d3_cold: pmrb.pme_support_d3_cold(),
                },
            },
            control: Control {
                power_state: pmrb.power_state(),
                reserved: pmrb.ctrl_reserved(),
                // PCI_PM_CTRL_NO_SOFT_RESET
                no_soft_reset: ((pmrb.ctrl_reserved() << 2) & 0x0008) != 0,
                pme_enabled: pmrb.ctrl_pme_enabled(),
                pme_status: pmrb.ctrl_pme_status(),
            },
            bridge: Bridge {
                reserved: pmrb.br_reserved(),
                b2_b3: pmrb.br_b2_b3(),
                bpcc_enabled: pmrb.br_bpcc_enabled(),
            },
            data: (pmrb.data_value() != 0).then(|| {
                Data {
                    value: pmrb.data_value(),
                    select: pmrb.data_select(),
                    scale: pmrb.data_scale(),
                }
            }),
            _view: Default::default(),
        }
    }
}
impl Display for PowerManagementInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (caps, ctrl, br, d) = (&self.capabilities, &self.control, &self.bridge, &self.data);
        match self._view.take() {
            View::Basic => {
                Ok(())
            },
            View::Lspci(verbose) => {
                write!(f, "Power Management version {}\n", caps.version)?;
                if verbose < 2 {
                    return Ok(())
                }
                write!(f, "\t\tFlags: PMEClk{} DSI{} D1{} D2{} AuxCurrent={} PME(D0{},D1{},D2{},D3hot{},D3cold{})\n",
                    CheckBox::lspci(caps.pme_clock), 
                    CheckBox::lspci(caps.device_specific_initialization), 
                    CheckBox::lspci(caps.d1_support), 
                    CheckBox::lspci(caps.d2_support), 
                    caps.aux_current, 
                    CheckBox::lspci(caps.pme_support.d0),
                    CheckBox::lspci(caps.pme_support.d1),
                    CheckBox::lspci(caps.pme_support.d2),
                    CheckBox::lspci(caps.pme_support.d3_hot),
                    CheckBox::lspci(caps.pme_support.d3_cold),
                )?;
                write!(f, "\t\tStatus: D{} NoSoftRst{} PME-Enable{} DSel={} DScale={} PME{}\n",
                    ctrl.power_state as usize,
                    CheckBox::lspci(ctrl.no_soft_reset),
                    CheckBox::lspci(ctrl.pme_enabled),
                    d.map(|d| d.select as usize).unwrap_or(0),
                    d.map(|d| d.scale as usize).unwrap_or(0),
                    CheckBox::lspci(ctrl.pme_status),
                )?;
                if br.bpcc_enabled || self.bridge.b2_b3 || self.bridge.reserved != 0 {
                    write!(f, "\t\tBridge: PM{} B3{}\n",
                        CheckBox::lspci(br.bpcc_enabled),
                        CheckBox::lspci(br.b2_b3),
                    )?;
                }
                Ok(())
            },
            View::Extended => {
                Ok(())
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use super::*;

    #[test]
    fn power_management_interface() {
        let data = [0x02,0x7e,0x00,0x00,0x40,0x00];
        // Capabilities: [c0] Power Management version 2
        //         Flags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)
        //         Status: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-
        //         Bridge: PM- B3+
        let result = PowerManagementInterface::from(data);
        let sample = PowerManagementInterface {
            capabilities: Capabilities {
                version: 0b10,
                pme_clock: false,
                reserved: false,
                device_specific_initialization: false,
                aux_current: AuxCurrent::SelfPowered,
                d1_support: true,
                d2_support: true,
                pme_support: PmeSupport {
                    d0: true,
                    d1: true,
                    d2: true,
                    d3_hot: true,
                    d3_cold: false,
                },
            },
            control: Control {
                power_state: PowerState::D0,
                reserved: 0b000000,
                no_soft_reset: false,
                pme_enabled: false,
                pme_status: false,
            },
            bridge: Bridge {
                reserved: 0,
                b2_b3: true,
                bpcc_enabled: false,
            },
            data: None,
            _view: Default::default(),
        };
        assert_eq!(sample, result);
    }

    #[test]
    fn display_lspci() {
        let data = [0x02,0x7e,0x00,0x00,0x40,0x00];
        let pmi = PowerManagementInterface::from(data);
        
        let v1_result = pmi.display(View::Lspci(1)).to_string();
        let v1_sample = "\
            Power Management version 2\n\
        ";
        assert_eq!(v1_sample, v1_result, "-v");
        
        let v2_result = pmi.display(View::Lspci(2)).to_string();
        let v2_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3+\n\
        ";
        assert_eq!(v2_sample, v2_result, "-vv");

        let v3_result = pmi.display(View::Lspci(3)).to_string();
        let v3_sample = "\
            Power Management version 2\n\
            \t\tFlags: PMEClk- DSI- D1+ D2+ AuxCurrent=0mA PME(D0+,D1+,D2+,D3hot+,D3cold-)\n\
            \t\tStatus: D0 NoSoftRst- PME-Enable- DSel=0 DScale=0 PME-\n\
            \t\tBridge: PM- B3+\n\
        ";
        assert_eq!(v3_sample, v3_result, "-vvv");
    }
}
