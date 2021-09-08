
use core::array;
use core::convert::TryFrom;

use serde::{Serialize, Deserialize,}; 
use bitfield_layout::{BitFieldLayout, Layout, };
use thiserror::Error;
use modular_bitfield::prelude::*;
use displaydoc::Display as DisplayDoc;

pub mod command;
pub use command::Command;

pub mod status;
pub use status::Status;

pub mod class_code;
pub use class_code::ClassCode;

pub mod bar;
pub use bar::{BaseAddresses,BaseAddress,HeaderBaseAdresses};

pub mod capabilities;
// pub use capabilities::Capabilities;



// #[bitfield(bits = 128)]
// #[derive(Debug, PartialEq, Eq,)] 
// struct CommonConfigurationSpace {
//     vendor_id: u16, 
//     device_id: u16,
//     command: u16, 
//     status: u16,
//     revision_id: u8,
//     class_code: B24, 
//     cache_line_size: u8, 
//     latency_timer: u8, 
//     header_type: u8,
//     bist: u8,
// }

// #[bitfield(bits = 384)]
// #[derive(Debug, PartialEq, Eq,)] 
// struct Type0ConfigurationSpace {
//     base_addresses: [u32; 6],
//     cardbus_cis_pointer: u32,
//     sub_vendor_id: u16,
//     sub_device_id: u16,
//     expansion_rom_base_address: u32,
//     capabilities_pointer: u8,
//     reserved: [u8; 7],
//     interrupt_line: u8, 
//     interrupt_pin: u8,
//     min_gnt: u8, 
//     max_lat: u8, 
// }


// #[bitfield(bits = 384)]
// #[derive(Debug, PartialEq, Eq,)] 
// struct Type1ConfigurationSpace {
//     base_addresses: [u32; 2],
//     primary_bus_number: u8,
//     secondary_bus_number: u8,
//     subordinate_bus_number: u8,
//     secondary_latency_timer: u8,
//     io_base: u8,
//     io_limit: u8,
//     secondary_status: u16,
//     memory_base: u16,
//     memory_limit: u16,
//     prefetchable_memory_base: u16,
//     prefetchable_memory_limit: u16,
//     prefetchable_memory_base_upper_32: u32,
//     prefetchable_memory_limit_upper_32: u32,
//     io_base_upper_16: u16,
//     io_limit_upper_16: u16,
//     capabilities_pointer: u8,
//     reserved: [u8; 3],
//     expansion_rom_base_address: u32,
//     interrupt_line: u8, 
//     interrupt_pin: u8,
//     bridge_control: u16,
// }

// #[bitfield(bits = 448)]
// #[derive(Debug, PartialEq, Eq,)] 
// struct Type2ConfigurationSpace {
//     base_addresses: [u32; 1],
//     capabilities_pointer: u8,
//     reserved: [u8; 1],
//     secondary_status: u16,
//     pci_bus_number: u8,
//     cardbus_bus_number: u8,
//     subordinate_bus_number: u8,
//     cardbus_latency_timer: u8,
//     memory_base_0: u32,
//     memory_limit_0: u32,
//     memory_base_1: u32,
//     memory_limit_1: u32,
//     io_base_0_lower: u16,
//     io_base_0_upper: u16,
//     io_limit_0_lower: u16,
//     io_limit_0_upper: u16,
//     io_base_1_lower: u16,
//     io_base_1_upper: u16,
//     io_limit_1_lower: u16,
//     io_limit_1_upper: u16,
//     interrupt_line: u8, 
//     interrupt_pin: u8,
//     bridge_control: u16,
//     subsystem_vendor_id: u16,
//     subsystem_id: u16,
//     legacy_mode_base_address: u32,
// }

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Capabilities(u8);

/// Identifies the layout of the second part of the predefined header (beginning at byte 10h in
/// Configuration Space) and also whether or not the device contains multiple functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Header {
    Normal(HeaderTypeNormal),
    Bridge(HeaderTypeBridge),
    Cardbus(HeaderTypeCardbus),
}

/// Common Header Fields
///
/// The field descriptions are common to all Header Types.
/// All PCI compliant devices must support the Vendor ID, Device ID, Command, Status,
/// Revision ID, Class Code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeaderCommon {
    /// Identifies the manufacturer of the device. Where valid IDs are allocated by PCI-SIG to
    /// ensure uniqueness and 0xFFFF is an invalid value that will be returned on read accesses to
    /// Configuration Space registers of non-existent devices. 
    pub vendor_id: u16, 
    /// Identifies the particular device. Where valid IDs are allocated by the vendor
    pub device_id: u16,
    pub command: Command, 
    pub status: Status,
    /// Device specific revision identifier.
    pub revision_id: u8,
    pub class_code: ClassCode, 
    /// Specifies the system cache line size in 32-bit units. A device can limit the number of
    /// cacheline sizes it can support, if a unsupported value is written to this field, the
    /// device will behave as if a value of 0 was written. 
    pub cache_line_size: u8, 
    /// Specifies the latency timer in units of PCI bus clocks.
    pub latency_timer: u8, 
    pub raw_header_type: RawHeaderType,
    pub bist: BuiltInSelfTest,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct RawHeaderType(u8);

/// Represents that status and allows control of a devices BIST (built-in self test).
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(from = "u8", into = "u8")]
pub struct BuiltInSelfTest {
    /// Device supports BIST
    pub is_capable: bool,
    /// When set to `true` the BIST is invoked. This bit is reset when BIST completes. If BIST does
    /// not complete after 2 seconds the device should be failed by system software.
    pub is_running: bool,
    /// Will return 0, after BIST execution, if the test completed successfully.
    pub completion_code: u8,
}

/// General device
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct HeaderTypeNormal {
    pub common: HeaderCommon,
    pub base_addresses: BaseAddresses<6>,
    /// Points to the Card Information Structure and is used by devices that share silicon between CardBus and PCI. 
    pub cardbus_cis_pointer: u32,
    /// Subsystem Vendor ID
    pub sub_vendor_id: u16,
    /// Subsystem Device ID
    pub sub_device_id: u16,
    /// Expansion ROM
    pub expansion_rom: ExpansionRom,
    pub capabilities: Capabilities,
    /// Reserved
    reserved: [u8; 7],
    /// Specifies which input of the system interrupt controllers the device's interrupt pin is
    /// connected to and is implemented by any device that makes use of an interrupt pin. For
    /// the x86 architecture this register corresponds to the PIC IRQ numbers 0-15 (and not I/O
    /// APIC IRQ numbers) and a value of 0xFF defines no connection.
    pub interrupt_line: u8, 
    pub interrupt_pin: InterruptPin,
    /// A read-only register that specifies the burst period length, in 1/4 microsecond units,
    /// that the device needs (assuming a 33 MHz clock rate).
    pub min_grant: u8, 
    /// A read-only register that specifies how often the device needs access to the PCI bus
    /// (in 1/4 microsecond units).
    pub max_latency: u8, 
}

/// PCI-to-PCI bridge
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct HeaderTypeBridge {
    pub common: HeaderCommon,
    /// Base Address Registers
    pub base_addresses: BaseAddresses<2>,
    /// Primary Bus Number
    pub primary_bus_number: u8,
    /// Secondary Bus Number
    pub secondary_bus_number: u8,
    /// Subordinate Bus Numbe
    pub subordinate_bus_number: u8,
    /// Secondary Latency Timer
    pub secondary_latency_timer: u8,
    /// I/O Base
    pub io_base: u8,
    /// I/O Limit
    pub io_limit: u8,
    /// Secondary Status
    pub secondary_status: u16,
    /// Memory Base
    pub memory_base: u16,
    /// Memory Limit
    pub memory_limit: u16,
    /// Prefetchable Memory Base
    pub prefetchable_memory_base: u16,
    /// Prefetchable Memory Limit 
    pub prefetchable_memory_limit: u16,
    /// Prefetchable Base Upper 32 Bits 
    pub prefetchable_memory_base_upper_32: u32,
    /// Prefetchable Limit Upper 32 Bits
    pub prefetchable_memory_limit_upper_32: u32,
    /// I/O Base Upper 16 Bits
    pub io_base_upper_16: u16,
    /// I/O Limit Upper 16 Bits
    pub io_limit_upper_16: u16,
    pub capabilities: Capabilities,
    /// Reserved
    reserved: [u8; 3],
    /// Expansion ROM
    pub expansion_rom: ExpansionRom,
    /// Specifies which input of the system interrupt controllers the device's interrupt pin is
    /// connected to and is implemented by any device that makes use of an interrupt pin. For
    /// the x86 architecture this register corresponds to the PIC IRQ numbers 0-15 (and not I/O
    /// APIC IRQ numbers) and a value of 0xFF defines no connection.
    pub interrupt_line: u8, 
    pub interrupt_pin: InterruptPin,
    pub bridge_control: BridgeControl,
}

/// PCI-to-CardBus bridge
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeaderTypeCardbus {
    pub common: HeaderCommon,
    /// PC Card Socket Status and Control Registers Base Address
    pub base_addresses: BaseAddresses<1>,
    pub capabilities: Capabilities,
    /// Reserved
    reserved: u8,
    /// Secondary status
    pub secondary_status: u16,
    /// PCI Bus Number
    pub pci_bus_number: u8,
    /// CardBus Bus Number
    pub cardbus_bus_number: u8,
    /// Subordinate Bus Number 
    pub subordinate_bus_number: u8,
    /// CardBus Latency Timer
    pub cardbus_latency_timer: u8,
    /// Memory Base #0
    pub memory_base_address_0: u32,
    /// Memory Limit #0
    pub memory_limit_address_0: u32,
    /// Memory Base Address #1
    pub memory_base_address_1: u32,
    /// Memory Limit #1
    pub memory_limit_address_1: u32,
    pub io_access_address_range_0: IoAccessAddressRange,
    pub io_access_address_range_1: IoAccessAddressRange,
    /// Specifies which input of the system interrupt controllers the device's interrupt pin is
    /// connected to and is implemented by any device that makes use of an interrupt pin. For
    /// the x86 architecture this register corresponds to the PIC IRQ numbers 0-15 (and not I/O
    /// APIC IRQ numbers) and a value of 0xFF defines no connection.
    pub interrupt_line: u8, 
    pub interrupt_pin: InterruptPin,
    pub bridge_control: CardbusBridgeControl,
    /// Subsystem Vendor ID
    pub subsystem_vendor_id: u16,
    /// Subsystem Device ID
    pub subsystem_device_id: u16,
    /// PC Card 16 Bit IF Legacy Mode Base Address
    pub legacy_mode_base_address: u32,
}

/// Specifies which interrupt pin the device uses.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(from = "u8", into = "u8")]
pub enum InterruptPin {
    Unused,
    IntA,
    IntB,
    IntC,
    IntD,
    Reserved(u8),
}




/// The Status register is used to record status information for PCI bus related events. Devices
/// may not need to implement all bits, depending on device functionality. Reserved bits should be
/// read-only and return zero when read. 
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SecondaryStatus(u16);

/// The Bridge Control register provides extensions to the Command register that are specific to a
/// PCI to PCI bridge.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct BridgeControl(u16);

/// The Bridge Control register provides extensions of the Command Register that are specific to PCI-to-CardBus bridges.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CardbusBridgeControl(u16);


#[derive(Debug, Error)]
pub enum HeaderTypeError {
    #[error("Can not deserialize")]
    BincodeDeserialize(#[from] bincode::Error),
    #[error("Short data length")]
    ShortLength,
}

pub struct View<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub lspci: &'a str,
}

/// The IO Base Register and I/O Limit Register defines the address range that is used by the
/// bridge to determine when to forward an I/O transaction to the CardBus. 
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(from = "[[u16; 2]; 2]", into = "[[u16; 2]; 2]")]
pub enum IoAccessAddressRange {
    Addr16Bit {
        base: u16,
        limit: u16
    },
    Addr32Bit {
        base: u32,
        limit: u32
    },
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(from = "u32", into = "u32")]
pub struct ExpansionRom {
    pub address: u32,
    pub is_enabled: bool,
    pub size: Option<u32>,
    pub io_flags: Option<u64>,
}



pub trait HeaderType {
    fn common(&self) -> &HeaderCommon;
    fn common_mut(&mut self) -> &mut HeaderCommon;
    fn interrupt_line(&self) -> u8;
    fn capabilities(&self) -> &Capabilities;
    fn capabilities_mut(&mut self) -> &mut Capabilities;
    fn base_addresses(&self) -> &dyn HeaderBaseAdresses;
    fn base_addresses_mut(&mut self) -> &mut dyn HeaderBaseAdresses;
}


impl Header {
    fn as_trait(&self) -> &dyn HeaderType {
        match self {
            Self::Normal(h) => h,
            Self::Bridge(h) => h,
            Self::Cardbus(h) => h,
        }
    }
    fn as_trait_mut(&mut self) -> &mut dyn HeaderType {
        match self {
            Self::Normal(h) => h,
            Self::Bridge(h) => h,
            Self::Cardbus(h) => h,
        }
    }
    pub fn common(&self) -> &HeaderCommon {
        self.as_trait().common()
    }
    pub fn common_mut(&mut self) -> &mut HeaderCommon {
        self.as_trait_mut().common_mut()
    }
    pub fn base_addresses(&self) -> &dyn HeaderBaseAdresses {
        self.as_trait().base_addresses()
    }
    pub fn base_addresses_mut(&mut self) -> &mut dyn HeaderBaseAdresses {
        self.as_trait_mut().base_addresses_mut()
    }
    pub fn capabilities(&self) -> &Capabilities {
        self.as_trait().capabilities()
    }
    pub fn capabilities_mut(&mut self) -> &mut Capabilities {
        self.as_trait_mut().capabilities_mut()
    }
    pub fn expansion_rom(&self) -> Option<&ExpansionRom> {
        match self {
            Self::Normal(h) => Some(&h.expansion_rom),
            Self::Bridge(h) => Some(&h.expansion_rom),
            Self::Cardbus(_) => None,
        }
    }

}

impl TryFrom<& [u8]> for Header {
    type Error = HeaderTypeError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 64 {
            Err(HeaderTypeError::ShortLength)
        } else {
            // Get the header type
            let header =
                match data[0x0E] & 0b0111_1111 {
                    0x00 => bincode::deserialize(&data).map(Header::Normal),
                    0x01 => bincode::deserialize(&data).map(Header::Bridge),
                    0x02 => 
                        if data.len() < 72 {
                            return Err(HeaderTypeError::ShortLength)
                        } else {
                            bincode::deserialize(&data).map(Header::Cardbus)
                        },
                            _ => unreachable!(),
                }
            .map_err(HeaderTypeError::BincodeDeserialize)?;
            Ok(header)
        }
    }
}

impl HeaderType for HeaderTypeNormal {
    fn common(&self) -> &HeaderCommon { &self.common }
    fn common_mut(&mut self) -> &mut HeaderCommon { &mut self.common }
    fn interrupt_line(&self) -> u8 { self.interrupt_line }
    fn capabilities(&self) -> &Capabilities { &self.capabilities }
    fn capabilities_mut(&mut self) -> &mut Capabilities { &mut self.capabilities }
    fn base_addresses(&self) -> &dyn HeaderBaseAdresses { &self.base_addresses }
    fn base_addresses_mut(&mut self) -> &mut dyn HeaderBaseAdresses { &mut self.base_addresses }
}

impl HeaderType for HeaderTypeBridge {
    fn common(&self) -> &HeaderCommon { &self.common }
    fn common_mut(&mut self) -> &mut HeaderCommon { &mut self.common }
    fn interrupt_line(&self) -> u8 { self.interrupt_line }
    fn capabilities(&self) -> &Capabilities { &self.capabilities }
    fn capabilities_mut(&mut self) -> &mut Capabilities { &mut self.capabilities }
    fn base_addresses(&self) -> &dyn HeaderBaseAdresses { &self.base_addresses }
    fn base_addresses_mut(&mut self) -> &mut dyn HeaderBaseAdresses { &mut self.base_addresses }
}

impl HeaderType for HeaderTypeCardbus {
    fn common(&self) -> &HeaderCommon { &self.common }
    fn common_mut(&mut self) -> &mut HeaderCommon { &mut self.common }
    fn interrupt_line(&self) -> u8 { self.interrupt_line }
    fn capabilities(&self) -> &Capabilities { &self.capabilities }
    fn capabilities_mut(&mut self) -> &mut Capabilities { &mut self.capabilities }
    fn base_addresses(&self) -> &dyn HeaderBaseAdresses { &self.base_addresses }
    fn base_addresses_mut(&mut self) -> &mut dyn HeaderBaseAdresses { &mut self.base_addresses }
}

/// To find [HeaderType] we should look inside HeaderType
impl RawHeaderType {
    /// A device that implements from two to eight functions. Each function has its own
    /// Configuration Space.
    pub fn has_multiple_functions(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }
}

impl Default for InterruptPin {
    fn default() -> Self { Self::Unused }
}
impl From<u8> for InterruptPin {
    fn from(data: u8) -> Self {
        match data {
            0x00 => Self::Unused,
            0x01 => Self::IntA,
            0x02 => Self::IntB,
            0x03 => Self::IntC,
            0x04 => Self::IntD,
            v => Self::Reserved(v),
        }
    }
}
impl From<InterruptPin> for u8 {
    fn from(pin: InterruptPin) -> Self {
        match pin {
            InterruptPin::Unused => 0x00,
            InterruptPin::IntA => 0x01,
            InterruptPin::IntB => 0x02,
            InterruptPin::IntC => 0x03,
            InterruptPin::IntD => 0x04,
            InterruptPin::Reserved(v) => v,
        }
    }
}

impl From<u8> for BuiltInSelfTest {
    fn from(data: u8) -> Self {
        Self {
            is_capable: data & 0b1000_0000 != 0,
            is_running: data & 0b0100_0000 != 0,
            completion_code: data & 0b1111,
        }
    }
}
impl From<BuiltInSelfTest> for u8 {
    fn from(bist: BuiltInSelfTest) -> Self {
        let mut result = bist.completion_code & 0b1111;
        if bist.is_capable {
            result |= 0b1000_0000;
        }
        if bist.is_running {
            result |= 0b0100_0000;
        }
        result
    }
}
        

impl SecondaryStatus {
    pub const LAYOUT: [View<'static>; 16] = [
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "66 MHz Capable", desc: "Does not apply to PCI Express and must be hardwired to 0", lspci: "66MHz" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Fast Back-to-Back Transactions Capable", desc: "Does not apply to PCI Express and must be hardwired to 0", lspci: "FastB2B" },
        View { name: "Master Data Parity Error", desc: "The bus agent asserted PERR# on a read or observed an assertion of PERR# on a write", lspci: "ParErr" },
        View { name: "DEVSEL Timing", desc: "Does not apply to PCI Express and must be hardwired to 0", lspci: "DEVSEL=medium" },
        View { name: "DEVSEL Timing", desc: "Does not apply to PCI Express and must be hardwired to 0", lspci: "DEVSEL=slow" },
        View { name: "Signalled Target Abort", desc: "Target device terminates a transaction with Target-Abort", lspci: ">TAbort" },
        View { name: "Received Target Abort", desc: "Master device transaction is terminated with Target-Abort", lspci: "<TAbort" },
        View { name: "Received Master Abort", desc: "Master device transaction (except for Special Cycle transactions) is terminated with Master-Abort", lspci: "<MAbort" },
        View { name: "Signalled System Error", desc: "Device asserts SERR#", lspci: ">SERR" },
        View { name: "Detected Parity Error", desc: "Device detects a parity error, even if parity error handling is disabled", lspci: "<PERR" },
    ];
}
impl Layout for SecondaryStatus {
    type Layout = array::IntoIter<View<'static>, 16>;
    fn layout() -> Self::Layout { array::IntoIter::new(Self::LAYOUT) }
}
impl BitFieldLayout for SecondaryStatus {
    type Value = u16;
    fn get(&self) -> Self::Value { self.0 }
    fn set(&mut self, new: Self::Value) { self.0 = new; }
}

impl BridgeControl {
    pub const LAYOUT: [View<'static>; 16] = [
        View { name: "Parity Error Response Enable", desc: "Controls the bridge’s response to address and data parity errors on the secondary interface", lspci: "Parity" },
        View { name: "SERR# Enable", desc: "Controls the forwarding of secondary interface SERR# assertions to the primary interface", lspci: "SERR" },
        View { name: "ISA Enable", desc: "Modifies the response by the bridge to ISA I/O addresses", lspci: "NoISA" },
        View { name: "VGA Enable", desc: "Modifies the response by the bridge to VGA compatible addresses", lspci: "VGA" },
        View { name: "VGA16", desc: "VGA16", lspci: "VGA16" },
        View { name: "Master-Abort Mode", desc: "Controls the behavior of a bridge when a Master-Abort termination occurs on either interface while the bridge is the master of the transaction", lspci: "MAbort" },
        View { name: "Secondary Bus Reset", desc: "Forces the assertion of RST# on the secondary interface", lspci: ">Reset" },
        View { name: "Fast Back-toBack Enable", desc: "Controls ability of the bridge to generate fast back-to-back transactions to different devices on the secondary interface", lspci: "FastB2B" },
        View { name: "Primary Discard Timer", desc: "Selects the number of PCI clocks that the bridge will wait for a master on the primary interface to repeat a Delayed Transaction request", lspci: "PriDiscTmr" },
        View { name: "Secondary Discard Timer", desc: "Selects the number of PCI clocks that the bridge will wait for a master on the secondary interface to repeat a Delayed Transaction request", lspci: "SecDiscTmr" },
        View { name: "Discard Timer Status", desc: "This bit is set when either the Primary Discard Timer or Secondary Discard Timer expires and a Delayed Completion is discarded from a queue in the bridge", lspci: "DiscTmrStat" },
        View { name: "Discard Timer SERR# Enable", desc: "This bit enables the bridge to assert SERR# on the primary interface when either the Primary Discard Timer or Secondary Discard Timer expires and a Delayed Transaction is discarded from a queue in the bridge", lspci: "DiscTmrSERREn" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
    ];
}
impl Layout for BridgeControl {
    type Layout = array::IntoIter<View<'static>, 16>;
    fn layout() -> Self::Layout { array::IntoIter::new(Self::LAYOUT) }
}
impl BitFieldLayout for BridgeControl {
    type Value = u16;
    fn get(&self) -> Self::Value { self.0 }
    fn set(&mut self, new: Self::Value) { self.0 = new; }
}

impl CardbusBridgeControl {
    pub const LAYOUT: [View<'static>; 16] = [
        View { name: "Parity Error Response Enable", desc: "Controls the response to parity errors on the CardBus", lspci: "Parity" },
        View { name: "SERR# Enable", desc: "Controls forwarding of SERR# signals indicated on the CardBus", lspci: "SERR" },
        View { name: "ISA Enable", desc: "This applies only to addresses that are enabled by the I/O Base and Limit registers and are also in the first 64 KBytes of PCI I/O space", lspci: "ISA" },
        View { name: "VGA Enable", desc: "Modifies the bridge's response to VGA compatible addresses", lspci: "VGA" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Master Abort Mode", desc: "Controls the behavior of the bridge when a master abort occurs on either PCI or CardBus interface when the bridge is master", lspci: "MAbort" },
        View { name: "CardBus Reset", desc: "When set the bridge will assert and hold CRST#", lspci: ">Reset" },
        View { name: "IREQ-INT Enable", desc: "When set this bit enables the IRQ routing register for 16-bit PC Cards", lspci: "16bInt" },
        View { name: "Memory 0 Prefetch Enable", desc: "When set enables Read prefetching from the memory window defined to by the Memory Base 0 and Memory Limit 0 registers", lspci: "Mem0Pref" },
        View { name: "Memory 1 Prefetch Enable", desc: "When set enables Read prefetching from the memory window defined to by the Memory Base 1 and Memory Limit 1 registers", lspci: "Mem1Pref" },
        View { name: "Write Posting Enable", desc: "Enables posting of Write data to and from the socket", lspci: "PostWrite" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
        View { name: "Reserved", desc: "Reserved", lspci: "Reserved" },
    ];
}
impl Layout for CardbusBridgeControl {
    type Layout = array::IntoIter<View<'static>, 16>;
    fn layout() -> Self::Layout { array::IntoIter::new(Self::LAYOUT) }
}
impl BitFieldLayout for CardbusBridgeControl {
    type Value = u16;
    fn get(&self) -> Self::Value { self.0 }
    fn set(&mut self, new: Self::Value) { self.0 = new; }
}

impl From<[[u16;2]; 2]> for IoAccessAddressRange {
    fn from(data: [[u16;2]; 2]) -> Self {
        let [[base_lower, base_upper], [limit_lower, limit_upper]] = data;
        match base_lower & 0b11 {
            0x00 => Self::Addr16Bit {
                base: base_lower & !0b11,
                limit: limit_lower
            },
            0x01 => {
                let base = u32::from_le_bytes({
                    let lower = (base_lower & !0b11).to_le_bytes();
                    let upper = base_upper.to_le_bytes();
                    [lower[0], lower[1], upper[0], upper[1]]
                });
                let limit = u32::from_le_bytes({
                    let lower = limit_lower.to_le_bytes();
                    let upper = limit_upper.to_le_bytes();
                    [lower[0], lower[1], upper[0], upper[1]]
                });
                Self::Addr32Bit { base, limit }
            },
            _ => Self::Unknown,
        }
    }
}
impl From<IoAccessAddressRange > for [[u16;2]; 2] {
    fn from(data: IoAccessAddressRange) -> Self {
        match data {
            IoAccessAddressRange::Addr16Bit { base, limit } => {
                [[base & !0b11, 0], [limit, 0]]
            },
            IoAccessAddressRange::Addr32Bit { base, limit } => {
                let base = base.to_le_bytes();
                let base_upper = u16::from_le_bytes([base[0] & !0b11, base[1]]);
                let base_lower = u16::from_le_bytes([base[2], base[3]]);
                let limit = limit.to_le_bytes();
                let limit_upper = u16::from_le_bytes([limit[0], limit[1]]);
                let limit_lower = u16::from_le_bytes([limit[2], limit[3]]);
                [[base_upper, base_lower], [limit_upper, limit_lower]]
            },
            _ => unreachable!(),
        }
    }
}
impl Default for IoAccessAddressRange {
    fn default() -> Self { IoAccessAddressRange::Unknown }
}
        
impl From<u32> for ExpansionRom {
    fn from(dword: u32) -> Self {
        Self { 
            address: dword & !0x7ff,
            is_enabled: dword & 1 != 0,
            ..Default::default()
        }
    }
}
impl From<ExpansionRom> for u32 {
    fn from(rom: ExpansionRom) -> Self {
        rom.address | (rom.is_enabled as u32)
    }
}



//#[cfg(test)]
//mod tests {
//    use pretty_assertions::assert_eq;
//    use super::*;

//    #[test]
//    fn io_access_address_range() {
//        let zeros = [[ 0x00, 0x00 ], [ 0x00, 0x00 ]];
//        assert_eq!(IoAccessAddressRange::Addr16Bit { base: 0, limit: 0 }, zeros.into(), "All zeros");

//        let a16 = [[ 0x50, 0x00 ], [ 0x60, 0x00 ]];
//        assert_eq!(IoAccessAddressRange::Addr16Bit { base: 0x50, limit: 0x60 }, a16.into(), "16 Bit");

//        let a32 = [[ 0x51, 0x50 ], [ 0x60, 0x60 ]];
//        assert_eq!(IoAccessAddressRange::Addr32Bit { base: 0x500050, limit: 0x600060 }, a32.into(), "32 Bit");

//        let unkn = [[ 0x52, 0x50 ], [ 0x60, 0x60 ]];
//        assert_eq!(IoAccessAddressRange::Unknown, unkn.into(), "Unknown");
//    }

//    #[test]
//    fn header_type_normal() {
//        // SATA controller [0106]: Intel Corporation Q170/Q150/B150/H170/H110/Z170/CM236 Chipset SATA Controller [AHCI Mode] [8086:a102] (rev 31) (prog-if 01 [AHCI 1.0])
//        // Subsystem: Dell Device [1028:06a5]
//        // Control: I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr+ Stepping- SERR+ FastB2B- DisINTx+
//        // Status: Cap+ 66MHz+ UDF- FastB2B+ ParErr- DEVSEL=medium >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
//        // Latency: 0
//        // Interrupt: pin A routed to IRQ 30
//        // Region 0: Memory at 93014000 (32-bit, non-prefetchable) [size=8K]
//        // Region 1: Memory at 93017000 (32-bit, non-prefetchable) [size=256]
//        // Region 2: I/O ports at 3040 [size=8]
//        // Region 3: I/O ports at 3048 [size=4]
//        // Region 4: I/O ports at 3020 [size=32]
//        // Region 5: Memory at 93016000 (32-bit, non-prefetchable) [size=2K]
//        // Capabilities: [80] MSI: Enable+ Count=1/1 Maskable- 64bit-
//        //         Address: fee003b8  Data: 0000
//        // Capabilities: [70] Power Management version 3
//        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=0mA PME(D0-,D1-,D2-,D3hot+,D3cold-)
//        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
//        // Capabilities: [a8] SATA HBA v1.0 BAR4 Offset=00000004
//        // Kernel driver in use: ahci
//        // Kernel modules: ahci
//        let data = [
//            0x86, 0x80, 0x02, 0xa1, 0x47, 0x05, 0xb0, 0x02, 0x31, 0x01, 0x06, 0x01, 0x00, 0x00, 0x00, 0x00,
//            0x00, 0x40, 0x01, 0x93, 0x00, 0x70, 0x01, 0x93, 0x41, 0x30, 0x00, 0x00, 0x49, 0x30, 0x00, 0x00,
//            0x21, 0x30, 0x00, 0x00, 0x00, 0x60, 0x01, 0x93, 0x00, 0x00, 0x00, 0x00, 0x28, 0x10, 0xa5, 0x06,
//            0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b, 0x01, 0x00, 0x00,
//        ];
//        let result: Header = Header::try_from(&data[..]).unwrap();
//        let sample = Header::Normal(
//            HeaderTypeNormal {
//                common: HeaderCommon {
//                    vendor_id: 0x8086,
//                    device_id: 0xa102,
//                    command: Command(0b0000_0101_0100_0111),
//                    status: Status(0b0000_0010_1011_0000),
//                    revision_id: 0x31,
//                    class_code: ClassCode {
//                        interface: 0x01,
//                        sub: 0x06,
//                        base: 0x01,
//                    },
//                    cache_line_size: 0,
//                    latency_timer: 0,
//                    raw_header_type: RawHeaderType(0),
//                    bist: BuiltInSelfTest {
//                        is_capable: false,
//                        is_running: false,
//                        completion_code: 0x00,
//                    },
//                },
//                base_addresses: BaseAddresses::Basic([
//                                    0x93014000,
//                                    0x93017000,
//                                    0x3041,
//                                    0x3049,
//                                    0x3021,
//                                    0x93016000
//                ]),
//                cardbus_cis_pointer: 0x00,
//                sub_vendor_id: 0x1028,
//                sub_device_id: 0x06a5,
//                expansion_rom: Default::default(),
//                capabilities: Capabilities(0x80),
//                reserved: [0,0,0,0,0,0,0],
//                interrupt_line: 0xb,
//                interrupt_pin: InterruptPin::IntA,
//                min_grant: 0,
//                max_latency: 0,
//            },
//        );
//        assert_eq!(sample, result);
//    }

//    #[test]
//    fn header_type_bridge() {
//        // PCI bridge [0604]: Renesas Technology Corp. SH7758 PCIe Switch [PS] [1912:001d] (prog-if 00 [Normal decode])
//        // Control: I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx-
//        // Status: Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
//        // Latency: 0
//        // BIST result: 00
//        // Bus: primary=04, secondary=05, subordinate=08, sec-latency=0
//        // I/O behind bridge: 0000f000-00000fff
//        // Memory behind bridge: 92000000-929fffff
//        // Prefetchable memory behind bridge: 0000000091000000-0000000091ffffff
//        // Secondary status: 66MHz- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- <SERR- <PERR-
//        // BridgeCtl: Parity+ SERR+ NoISA- VGA+ MAbort- >Reset- FastB2B-
//        //         PriDiscTmr- SecDiscTmr- DiscTmrStat- DiscTmrSERREn-
//        // Capabilities: [40] Power Management version 3
//        //         Flags: PMEClk- DSI- D1- D2- AuxCurrent=0mA PME(D0+,D1-,D2-,D3hot+,D3cold+)
//        //         Status: D0 NoSoftRst+ PME-Enable- DSel=0 DScale=0 PME-
//        // Capabilities: [50] MSI: Enable- Count=1/1 Maskable- 64bit+
//        //         Address: 0000000000000000  Data: 0000
//        // Capabilities: [70] Express (v2) Upstream Port, MSI 00
//        //         DevCap: MaxPayload 128 bytes, PhantFunc 0
//        //                 ExtTag+ AttnBtn- AttnInd- PwrInd- RBE+ SlotPowerLimit 0.000W
//        //         DevCtl: Report errors: Correctable- Non-Fatal+ Fatal+ Unsupported+
//        //                 RlxdOrd+ ExtTag+ PhantFunc- AuxPwr- NoSnoop+
//        //                 MaxPayload 128 bytes, MaxReadReq 128 bytes
//        //         DevSta: CorrErr+ UncorrErr- FatalErr- UnsuppReq+ AuxPwr- TransPend-
//        //         LnkCap: Port #0, Speed 2.5GT/s, Width x1, ASPM L0s, Exit Latency L0s unlimited, L1 unlimited
//        //                 ClockPM- Surprise- LLActRep- BwNot- ASPMOptComp+
//        //         LnkCtl: ASPM Disabled; Disabled- CommClk+
//        //                 ExtSynch- ClockPM- AutWidDis- BWInt- AutBWInt-
//        //         LnkSta: Speed 2.5GT/s, Width x1, TrErr- Train- SlotClk+ DLActive- BWMgmt- ABWMgmt-
//        //         DevCap2: Completion Timeout: Not Supported, TimeoutDis-, LTR-, OBFF Not Supported
//        //         DevCtl2: Completion Timeout: 50us to 50ms, TimeoutDis-, LTR-, OBFF Disabled
//        //         LnkCtl2: Target Link Speed: 2.5GT/s, EnterCompliance- SpeedDis-
//        //                  Transmit Margin: Normal Operating Range, EnterModifiedCompliance- ComplianceSOS-
//        //                  Compliance De-emphasis: -6dB
//        //         LnkSta2: Current De-emphasis Level: -6dB, EqualizationComplete-, EqualizationPhase1-
//        //                  EqualizationPhase2-, EqualizationPhase3-, LinkEqualizationRequest-
//        // Capabilities: [b0] Subsystem: Renesas Technology Corp. SH7758 PCIe Switch [PS] [1912:001d]
//        // Capabilities: [100 v1] Advanced Error Reporting
//        //         UESta:  DLP- SDES- TLP- FCP- CmpltTO- CmpltAbrt- UnxCmplt- RxOF- MalfTLP- ECRC- UnsupReq- ACSViol-
//        //         UEMsk:  DLP- SDES- TLP- FCP- CmpltTO- CmpltAbrt+ UnxCmplt+ RxOF- MalfTLP- ECRC- UnsupReq- ACSViol-
//        //         UESvrt: DLP+ SDES+ TLP+ FCP+ CmpltTO- CmpltAbrt- UnxCmplt- RxOF+ MalfTLP+ ECRC+ UnsupReq- ACSViol-
//        //         CESta:  RxErr- BadTLP- BadDLLP- Rollover- Timeout- NonFatalErr+
//        //         CEMsk:  RxErr+ BadTLP+ BadDLLP+ Rollover+ Timeout+ NonFatalErr+
//        //         AERCap: First Error Pointer: 00, GenCap+ CGenEn- ChkCap+ ChkEn-
//        // Kernel driver in use: pcieport
//        let data = [
//            0x12, 0x19, 0x1d, 0x00, 0x07, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0x06, 0x00, 0x00, 0x01, 0x80,
//            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x05, 0x08, 0x00, 0xf1, 0x01, 0x00, 0x00,
//            0x00, 0x92, 0x90, 0x92, 0x01, 0x91, 0xf1, 0x91, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x1b, 0x00,
//        ];
//        let result: Header = Header::try_from(&data[..]).unwrap();
//        println!("{:#04X?}", &result);
//        let sample = Header::Bridge(
//            HeaderTypeBridge {
//                common: HeaderCommon {
//                    vendor_id: 0x1912,
//                    device_id: 0x001d,
//                    command: Command(0b0000_0000_0000_0111),
//                    status: Status(0b0000_0000_0001_0000),
//                    revision_id: 0x00,
//                    class_code: ClassCode {
//                        interface: 0x00,
//                        sub: 0x04,
//                        base: 0x06,
//                    },
//                    cache_line_size: 0,
//                    latency_timer: 0,
//                    raw_header_type: RawHeaderType(0x01),
//                    bist: BuiltInSelfTest {
//                        is_capable: true,
//                        is_running: false,
//                        completion_code: 0x00,
//                    },
//                },
//                base_addresses: BaseAddresses::Basic([0; 2]),
//                primary_bus_number: 0x04,
//                secondary_bus_number: 0x05,
//                subordinate_bus_number: 0x08,
//                secondary_latency_timer: 0x00,
//                io_base: 0xf1,
//                io_limit: 0x01,
//                secondary_status: 0x0000,
//                memory_base: (0x92000000u32 >> 16) as u16,
//                memory_limit: (0x929fffffu32 - 0xfffff >> 16) as u16,
//                prefetchable_memory_base: (0x91000000u64 >> 16) as u16 + 0x01,
//                prefetchable_memory_limit: (0x91ffffffu64 - 0xfffff >> 16) as u16 + 0x01,
//                prefetchable_memory_base_upper_32: 0x00000000,
//                prefetchable_memory_limit_upper_32: 0x00000000,
//                io_base_upper_16: 0x00,
//                io_limit_upper_16: 0x00,
//                capabilities: Capabilities(0x40),
//                reserved: [0x00; 3],
//                expansion_rom: Default::default(),
//                interrupt_line: 0xff, 
//                interrupt_pin: InterruptPin::Unused,
//                bridge_control: BridgeControl(0b0000_0000_0001_1011),
//            },
//        );
//        assert_eq!(sample, result);
//    }

//    #[test]
//    fn header_type_cardbus() {
//        // Random data with some bytes fixed
//        //
//        // CardBus bridge [0607]: Device [df8e:05ee] (rev 37)
//        // Subsystem: Device [3322:5544]
//        // Control: I/O- Mem- BusMaster+ SpecCycle- MemWINV+ VGASnoop+ ParErr- Stepping+ SERR- FastB2B- DisINTx-
//        // Status: Cap+ 66MHz+ UDF+ FastB2B- ParErr+ DEVSEL=medium >TAbort+ <TAbort- <MAbort- >SERR+ <PERR- INTx+
//        // Latency: 41, Cache Line Size: 968 bytes
//        // Interrupt: pin Z routed to IRQ 6
//        // Region 0: Memory at 35f88000 (32-bit, non-prefetchable) [disabled]
//        // Bus: primary=6d, secondary=ba, subordinate=fe, sec-latency=252
//        // Memory window 0: 11f54000-22475fff [disabled] (prefetchable)
//        // Memory window 1: 33853000-44d0cfff [disabled]
//        // I/O window 0: 00000060-00000073 [disabled]
//        // I/O window 1: 00060060-00070073 [disabled]
//        // BridgeCtl: Parity+ SERR- ISA+ VGA- MAbort- >Reset+ 16bInt- PostWrite+
//        // 16-bit legacy interface ports at 3322
//        let data = [
//            0x8e, 0xdf, 0xee, 0x05, 0xb4, 0x00, 0x78, 0x4b, 0x37, 0x00, 0x07, 0x06, 0xf2, 0x29, 0x82, 0x00,
//            0x00, 0x80, 0xf8, 0x35, 0x80, 0x00, 0x00, 0x00, 0x6d, 0xba, 0xfe, 0xfc, 0x00, 0x40, 0xf5, 0x11,
//            0x00, 0x50, 0x47, 0x22, 0x00, 0x30, 0x85, 0x33, 0x00, 0xc0, 0xd0, 0x44, 0x60, 0x00, 0x00, 0x00,
//            0x70, 0x00, 0x00, 0x00, 0x61, 0x00, 0x06, 0x00, 0x70, 0x00, 0x07, 0x00, 0x06, 0x1a, 0x45, 0x05,
//            0x22, 0x33, 0x44, 0x55, 0x22, 0x33, 0x00, 0x00,
//        ];
//        let result: Header = Header::try_from(&data[..]).unwrap();
//        println!("{:02X?}", &data);
//        let sample = Header::Cardbus(
//            HeaderTypeCardbus {
//                common: HeaderCommon {
//                    vendor_id: 0xdf8e,
//                    device_id: 0x05ee,
//                    command: Command(0b0000_0000_1011_0100),
//                    status: Status(0b0100_1011_0111_1000),
//                    revision_id: 0x37,
//                    class_code: ClassCode {
//                        interface: 0x00,
//                        sub: 0x07,
//                        base: 0x06,
//                    },
//                    cache_line_size: 0xf2,
//                    latency_timer: 41,
//                    raw_header_type: RawHeaderType(0x82),
//                    bist: BuiltInSelfTest {
//                        is_capable: false,
//                        is_running: false,
//                        completion_code: 0x00,
//                    },
//                },
//                base_addresses: BaseAddresses::Basic([0x35f88000]),
//                capabilities: Capabilities(0x80),
//                reserved: 0x00,
//                secondary_status: 0x0000,
//                pci_bus_number: 0x6d,
//                cardbus_bus_number: 0xba,
//                subordinate_bus_number: 0xfe,
//                cardbus_latency_timer: 252,
//                memory_base_address_0: 0x11f54000,
//                memory_limit_address_0: 0x22475fff - 0xfff,
//                memory_base_address_1: 0x33853000,
//                memory_limit_address_1: 0x44d0cfff - 0xfff,
//                io_access_address_range_0: IoAccessAddressRange::Addr16Bit {
//                    base: 0x0060,
//                    // Don't know why '+ 3' here https://github.com/pciutils/pciutils/blob/5bdf63b6b1bc35b59c4b3f47f7ca83ca1868155b/lspci.c#L683
//                    limit: 0x0073 - 3,
//                },
//                io_access_address_range_1: IoAccessAddressRange::Addr32Bit {
//                    base: 0x00060060,
//                    // Don't know why '+ 3' here https://github.com/pciutils/pciutils/blob/5bdf63b6b1bc35b59c4b3f47f7ca83ca1868155b/lspci.c#L683
//                    limit: 0x00070073 - 3,
//                },
//                interrupt_line: 0x06, 
//                interrupt_pin: InterruptPin::Reserved(0x1a),
//                bridge_control: CardbusBridgeControl(0b0000_0101_0100_0101),
//                subsystem_vendor_id: 0x3322,
//                subsystem_device_id: 0x5544,
//                legacy_mode_base_address: 0x3322,
//            },
//        );
//        assert_eq!(sample, result);
//    }

//    #[test]
//    fn text_header_type_bridge() {
//        // PCI bridge [0604]: Renesas Technology Corp. SH7758 PCIe Switch [PS] [1912:001d] (prog-if 00 [Normal decode])
//        // Control: I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx-
//        // Status: Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-
//        // Latency: 0
//        // BIST result: 00
//        // Bus: primary=04, secondary=05, subordinate=08, sec-latency=0
//        // Memory behind bridge: 92000000-929fffff
//        // Prefetchable memory behind bridge: 0000000091000000-0000000091ffffff
//        // Secondary status: 66MHz- FastB2B- ParErr- DEVSEL=fast >TAbort- <TAbort- <MAbort- <SERR- <PERR-
//        // BridgeCtl: Parity+ SERR+ NoISA- VGA+ MAbort- >Reset- FastB2B-
//        //         PriDiscTmr- SecDiscTmr- DiscTmrStat- DiscTmrSERREn-
//        // Capabilities: <access denied>
//        // Kernel driver in use: pcieport
//        let data = [
//            0x12, 0x19, 0x1d, 0x00, 0x07, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0x06, 0x00, 0x00, 0x01, 0x80,
//            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x05, 0x08, 0x00, 0xf1, 0x01, 0x00, 0x00,
//            0x00, 0x92, 0x90, 0x92, 0x01, 0x91, 0xf1, 0x91, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x1b, 0x00,
//        ];
//        let decoded: HeaderTypeBridge = bincode::deserialize(&data[..]).unwrap();
//        println!("{:#04X?}", &decoded);

//        assert_eq!(
//            ("Bridge", Some("PCI bridge"), Some("Normal decode")),
//            decoded.common.class_code.meaning(),
//            "PCI bridge [0604]"
//        );
//        assert_eq!(0x1912, decoded.common.vendor_id, "Renesas Technology Corp.");
//        assert_eq!(0x001d, decoded.common.device_id, "SH7758 PCIe Switch [PS]");
//        assert_eq!(0x00, decoded.common.class_code.interface, "prog-if 00 [Normal decode]");

//        let command_result = decoded.common.command.flags()
//            .take(11)
//            .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
//            .collect::<Vec<String>>()
//            .join(" ");
//        assert_eq!(
//            "I/O+ Mem+ BusMaster+ SpecCycle- MemWINV- VGASnoop- ParErr- Stepping- SERR- FastB2B- DisINTx-",
//            command_result,
//            "Control"
//        );

//        let status_result = {
//            let v = decoded.common.status.flags()
//                .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
//                .collect::<Vec<String>>();
//            format!("{} {}", &v[4..].join(" "), v[3])
//        };
//        assert_eq!(
//            "Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=medium- DEVSEL=slow- >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-",
//            status_result,
//            "Status"
//        );

//        assert_eq!(0x00, decoded.common.latency_timer, "Latency");

//        let bist_result =
//            if decoded.common.bist.is_capable {
//                if decoded.common.bist.is_running {
//                    Err("BIST is running")
//                } else {
//                    Ok(decoded.common.bist.completion_code)
//                }
//            } else {
//                Err("Not capable")
//            };
//        assert_eq!(Ok(0x00), bist_result, "BIST");

//        assert_eq!(
//            (0x04, 0x05, 0x08, 0x00),
//            (decoded.primary_bus_number, decoded.secondary_bus_number,
//             decoded.subordinate_bus_number, decoded.secondary_latency_timer),
//            "Bus: primary=04, secondary=05, subordinate=08, sec-latency=0"
//        );
//        assert_eq!(
//            (0x92000000u32, 0x929fffffu32),
//            ((decoded.memory_base as u32) << 16, ((decoded.memory_limit as u32) << 16) + 0xfffff),
//            "Memory behind bridge: 92000000-929fffff"
//        );
//        assert_eq!(
//            (0x91000000u64, 0x91ffffffu64),
//            ((decoded.prefetchable_memory_base as u64 & !0x0f) << 16, ((decoded.prefetchable_memory_limit as u64 & !0x0f) << 16) + 0xfffff),
//            "Prefetchable memory behind bridge: 0000000091000000-0000000091ffffff" 
//        );

//        let secondary_status_result = {
//            let v = decoded.common.status.flags()
//                .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
//                .collect::<Vec<String>>();
//            format!("{} {}", &v[4..].join(" "), v[3])
//        };
//        assert_eq!(
//            "Cap+ 66MHz- UDF- FastB2B- ParErr- DEVSEL=medium- DEVSEL=slow- >TAbort- <TAbort- <MAbort- >SERR- <PERR- INTx-",
//            secondary_status_result,
//            "Secondary status"
//        );
            
//        let bridge_control_result = decoded.bridge_control.flags()
//            .take(12)
//            .map(|f| format!("{}{}", f.value.lspci, if f.is_set { "+" } else { "-" }))
//            .collect::<Vec<String>>()
//            .join(" ");
//        assert_eq!(
//            "Parity+ SERR+ NoISA- VGA+ VGA16+ MAbort- >Reset- FastB2B- PriDiscTmr- SecDiscTmr- DiscTmrStat- DiscTmrSERREn-",
//            bridge_control_result,
//            "Bridge control"
//        );
//    }

//}
