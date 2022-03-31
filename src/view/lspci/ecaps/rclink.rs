use core::fmt;

use pcics::extended_capabilities::root_complex_link_declaration::{
    RootComplexLinkDeclaration, ElementType, LinkEntry, LinkAddress,
};

use crate::view::{DisplayMultiViewBasic, MultiView, Verbose, BoolView};



impl<'a> DisplayMultiViewBasic<Verbose> for RootComplexLinkDeclaration<'a> {}
impl<'a> fmt::Display for MultiView<&'a RootComplexLinkDeclaration<'a,>, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.view.0;
        writeln!(f, "Root Complex Link")?;
        if verbose < 2 {
            return Ok(());
        }
        let RootComplexLinkDeclaration {
            element_self_description,
            link_entries,
        } = self.data;
        writeln!(f,
            "\t\tDesc:\tPortNumber={:02x} ComponentID={:02x} EltType={}", 
            element_self_description.port_number,
            element_self_description.component_id,
            element_self_description.element_type.display(()),
        )?;
        for (n, link_entry) in link_entries.clone().enumerate() {
            write!(f, "\t\tLink{}:\t", n)?;
            let LinkEntry { link_description, link_address, } = link_entry;
            writeln!(f,
                "Desc:\tTargetPort={:02x} TargetComponent={:02x} AssocRCRB{} LinkType={} LinkValid{}", 
                link_description.target_port_number,
                link_description.target_component_id,
                link_description.associate_rcrb_header.display(BoolView::PlusMinus),
                if link_description.link_type != 0 { "Config" } else { "MemMapped" },
                link_description.link_valid.display(BoolView::PlusMinus),
            )?;
            match link_address {
                LinkAddress::MemoryMappedSpace(v) => writeln!(f, "\t\t\tAddr:\t{:016x}", v),
                LinkAddress::ConfigurationSpace { function, device, bus, address, .. } =>
                    writeln!(f,
                        "\t\t\tAddr:\t{:02x}:{:02x}.{}  CfgSpace={:016x}",
                        bus, device, function, address
                    ),
            }?;
        }
        Ok(())
    }
}


impl<'a> DisplayMultiViewBasic<()> for ElementType {}
impl<'a> fmt::Display for MultiView<&'a ElementType, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ElementType::ConfigurationSpaceElement => write!(f, "Config"),
            ElementType::SystemEgressPortOrInternalSink => write!(f, "Egress"),
            ElementType::InternalRootComplexLink => write!(f, "Internal"),
            _ => Ok(()),
        }
    }
}
