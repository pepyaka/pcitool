use core::fmt;

use pcics::capabilities::enhanced_allocation::{
    BarEquivalentIndicator, EnhancedAllocation, EnhancedAllocationEntry, ResourceDefinition,
    ResourceRangeAddress, Type1SecondDw,
};

use super::{Flag, Verbose};

impl<'a> fmt::Display for Verbose<&'a EnhancedAllocation<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let EnhancedAllocation {
            num_entries,
            type_1_second_dw,
            entries,
        } = self.data;
        let verbose = self.verbose;
        write!(f, "Enhanced Allocation (EA): NumEntries={}", num_entries)?;
        if let Some(Type1SecondDw {
            fixed_secondary_bus_number,
            fixed_subordinate_bus_number,
        }) = type_1_second_dw
        {
            write!(
                f,
                ", secondary={}, subordinate={}",
                fixed_secondary_bus_number, fixed_subordinate_bus_number
            )?;
        }
        writeln!(f)?;
        if verbose < 2 {
            return Ok(());
        }
        for (n, entry) in entries.clone().enumerate() {
            let EnhancedAllocationEntry {
                entry_size,
                bar_equivalent_indicator,
                primary_properties,
                secondary_properties,
                writable,
                enable,
                base,
                max_offset,
            } = entry;
            writeln!(
                f,
                "\t\tEntry {}: Enable{} Writable{} EntrySize={}",
                n,
                Flag(enable),
                Flag(writable),
                entry_size
            )?;
            let bei = match bar_equivalent_indicator {
                BarEquivalentIndicator::Location10h => "BAR 0",
                BarEquivalentIndicator::Location14h => "BAR 1",
                BarEquivalentIndicator::Location18h => "BAR 2",
                BarEquivalentIndicator::Location1Ch => "BAR 3",
                BarEquivalentIndicator::Location20h => "BAR 4",
                BarEquivalentIndicator::Location24h => "BAR 5",
                BarEquivalentIndicator::BehindType1Function => "resource behind function",
                BarEquivalentIndicator::EquivalentNotIndicated => "not indicated",
                BarEquivalentIndicator::ExpansionRomBaseAddress => "expansion ROM",
                BarEquivalentIndicator::VfBar0 => "VF-BAR 0",
                BarEquivalentIndicator::VfBar1 => "VF-BAR 1",
                BarEquivalentIndicator::VfBar2 => "VF-BAR 2",
                BarEquivalentIndicator::VfBar3 => "VF-BAR 3",
                BarEquivalentIndicator::VfBar4 => "VF-BAR 4",
                BarEquivalentIndicator::VfBar5 => "VF-BAR 5",
                BarEquivalentIndicator::Reserved => "reserved",
            };
            writeln!(f, "\t\t\t BAR Equivalent Indicator: {}", bei)?;
            writeln!(
                f,
                "\t\t\t PrimaryProperties: {}",
                ResourceDefinitionView {
                    rd: &primary_properties,
                    is_secondary: false
                },
            )?;
            writeln!(
                f,
                "\t\t\t SecondaryProperties: {}",
                ResourceDefinitionView {
                    rd: &secondary_properties,
                    is_secondary: true
                },
            )?;
            writeln!(f, "\t\t\t Base: {}", ResourceRangeAddressView(base))?;
            writeln!(
                f,
                "\t\t\t MaxOffset: {}",
                ResourceRangeAddressView(max_offset)
            )?;
        }

        Ok(())
    }
}

struct ResourceDefinitionView<'a> {
    rd: &'a ResourceDefinition,
    is_secondary: bool,
}

impl<'a> fmt::Display for ResourceDefinitionView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let &ResourceDefinitionView { rd, is_secondary } = self;
        match rd {
            ResourceDefinition::MemorySpaceNonPrefetchable => {
                write!(f, "memory space, non-prefetchable")
            }
            ResourceDefinition::MemorySpacePrefetchable => write!(f, "memory space, prefetchable"),
            ResourceDefinition::IoSpace => write!(f, "I/O space"),
            ResourceDefinition::PfForVfMemorySpacePrefetchable => {
                write!(f, "VF memory space, prefetchable")
            }
            ResourceDefinition::PfForVfMemorySpaceNonPrefetchable => {
                write!(f, "VF memory space, non-prefetchable")
            }
            ResourceDefinition::Type1ForAbbMemoryNonPrefetchable => {
                write!(f, "allocation behind bridge, non-prefetchable memory")
            }
            ResourceDefinition::Type1ForAbbMemoryPrefetchable => {
                write!(f, "allocation behind bridge, prefetchable memory")
            }
            ResourceDefinition::Type1ForAbbIoSpace => {
                write!(f, "allocation behind bridge, I/O space")
            }
            ResourceDefinition::Reserved(v) => write!(f, "[{:02x}]", v),
            ResourceDefinition::UnavailableMemorySpace => {
                write!(f, "memory space resource unavailable for use")
            }
            ResourceDefinition::UnavailableIoSpace => {
                write!(f, "I/O space resource unavailable for use")
            }
            ResourceDefinition::Unavailable => {
                if is_secondary {
                    write!(
                        f,
                        "entry unavailable for use, PrimaryProperties should be used"
                    )
                } else {
                    write!(f, "entry unavailable for use")
                }
            }
        }
    }
}

struct ResourceRangeAddressView(ResourceRangeAddress);

impl fmt::Display for ResourceRangeAddressView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ResourceRangeAddress::U32(v) => write!(f, "{:08x}", v),
            ResourceRangeAddress::U64(v) => write!(f, "{:08x}", v),
        }
    }
}
