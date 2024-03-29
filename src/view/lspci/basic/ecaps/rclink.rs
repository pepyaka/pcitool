use core::fmt;

use pcics::extended_capabilities::root_complex_link_declaration::{
    ElementSelfDescription, ElementType, LinkAddress, LinkEntriesState, LinkEntry,
    RootComplexLinkDeclaration, RootComplexLinkDeclarationError,
};

use crate::view::{DisplayMultiView, MultiView, };

use super::{Verbose, Flag};

impl<'a> fmt::Display for Verbose<&'a RootComplexLinkDeclaration<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let RootComplexLinkDeclaration {
            element_self_description,
            link_entries,
        } = self.data;
        let verbose = self.verbose;
        writeln!(f, "Root Complex Link")?;
        if verbose < 2 {
            return Ok(());
        }
        fmt_esd(f, element_self_description)?;
        for (n, link_entry) in link_entries.clone().enumerate() {
            write!(f, "\t\tLink{}:\t", n)?;
            let LinkEntry {
                link_description,
                link_address,
            } = link_entry;
            writeln!(
                f,
                "Desc:\tTargetPort={:02x} TargetComponent={:02x} AssocRCRB{} LinkType={} LinkValid{}",
                link_description.target_port_number,
                link_description.target_component_id,
                Flag(link_description .associate_rcrb_header ),
                if link_description.link_type != 0 {
                    "Config"
                } else {
                    "MemMapped"
                },
                Flag(link_description.link_valid),
            )?;
            match link_address {
                LinkAddress::MemoryMappedSpace(address) => {
                    writeln!(f, "\t\t\tAddr:\t{:016x}", address)
                }
                LinkAddress::ConfigurationSpace {
                    function,
                    device,
                    bus,
                    address,
                    ..
                } => writeln!(
                    f,
                    "\t\t\tAddr:\t{:02x}:{:02x}.{}  CfgSpace={:016x}",
                    bus,
                    device,
                    function,
                    // [BUG]
                    // https://github.com/pciutils/pciutils/blob/864aecdea9c7db626856d8d452f6c784316a878c/ls-ecaps.c#L630
                    // addr_lo showed as is
                    if cfg!(feature = "ls_ecaps_rclink_cfgspace") {
                        let [addr_l, addr_h]: [u32; 2] = link_address.into();
                        (addr_h as u64) << 32 | (addr_l as u64)
                    } else {
                        address
                    }
                ),
            }?;
        }
        if let LinkEntriesState::Incomplete | LinkEntriesState::Invalid = link_entries.state {
            writeln!(f, "\t\tLink{}:\t<unreadable>", link_entries.clone().count())?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Verbose<&'a RootComplexLinkDeclarationError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
        writeln!(f, "Root Complex Link")?;
        if verbose < 2 {
            return Ok(());
        }
        match self.data {
            esd @ RootComplexLinkDeclarationError::ElementSelfDescription => writeln!(f, "{}", esd),
            RootComplexLinkDeclarationError::NumberOfLinkEntries {
                element_self_description,
            } => fmt_esd(f, element_self_description),
            RootComplexLinkDeclarationError::ReservedSpace {
                element_self_description,
            } => fmt_esd(f, element_self_description),
            RootComplexLinkDeclarationError::LinkEntry1 {
                element_self_description,
            } => fmt_esd(f, element_self_description),
        }
    }
}

impl DisplayMultiView<()> for ElementType {}
impl<'a> fmt::Display for MultiView<&'a ElementType, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ElementType::ConfigurationSpaceElement => write!(f, "Config"),
            ElementType::SystemEgressPortOrInternalSink => write!(f, "Egress"),
            ElementType::InternalRootComplexLink => write!(f, "Internal"),
            ElementType::Reserved(v) => write!(f, "??{}", v),
        }
    }
}

fn fmt_esd(f: &mut fmt::Formatter<'_>, esd: &ElementSelfDescription) -> fmt::Result {
    writeln!(
        f,
        "\t\tDesc:\tPortNumber={:02x} ComponentID={:02x} EltType={}",
        esd.port_number,
        esd.component_id,
        // [BUG]
        // https://github.com/pciutils/pciutils/blob/864aecdea9c7db626856d8d452f6c784316a878c/ls-ecaps.c#L596
        // takes 8 bits, should be 4
        if cfg!(feature = "ls_ecaps_rclink_eltype") {
            match esd.reserved << 4 | u8::from(esd.element_type.clone()) {
                0 => "Config".to_string(),
                1 => "Egress".to_string(),
                2 => "Internal".to_string(),
                v => format!("??{}", v),
            }
        } else {
            format!("{}", esd.element_type.display(()))
        },
    )
}
