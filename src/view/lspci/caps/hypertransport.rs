use core::fmt;

use pcics::capabilities::hypertransport::{
    Hypertransport, HostOrSecondaryInterface, SlaveOrPrimaryInterface, MsiMapping, LinkWidth, LinkFrequency
};

use crate::view::{Verbose, DisplayMultiViewBasic, MultiView, BoolView};

impl<'a> DisplayMultiViewBasic<Verbose> for Hypertransport {}
impl<'a> fmt::Display for MultiView<&'a Hypertransport, Verbose> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Verbose(verbose) = self.view;
        match self.data {
            Hypertransport::SlaveOrPrimaryInterface(sopi) =>
                fmt_pri(f, sopi, verbose),
            Hypertransport::HostOrSecondaryInterface(hosi) =>
                fmt_sec(f, hosi, verbose),
            Hypertransport::Switch(_) =>
                writeln!(f, "HyperTransport: Switch"),
            Hypertransport::InterruptDiscoveryAndConfiguration(_) =>
                writeln!(f, "HyperTransport: Interrupt Discovery and Configuration"),
            Hypertransport::RevisionId(id) =>
                  writeln!(f, "HyperTransport: Revision ID: {}.{:02}", id.major, id.minor),
            Hypertransport::UnitIdClumping(_) =>
                writeln!(f, "HyperTransport: UnitID Clumping"),
            Hypertransport::ExtendedConfigurationSpaceAccess(_) =>
                writeln!(f, "HyperTransport: Extended Configuration Space Access"),
            Hypertransport::AddressMapping(_) =>
                writeln!(f, "HyperTransport: Address Mapping"),
            Hypertransport::MsiMapping(msim) =>
                fmt_msim(f, msim, verbose),
            Hypertransport::DirectRoute(_) =>
                writeln!(f, "HyperTransport: DirectRoute"),
            Hypertransport::VCSet(_) =>
                writeln!(f, "HyperTransport: VCSet"),
            Hypertransport::RetryMode(_) =>
                writeln!(f, "HyperTransport: Retry Mode"),
            Hypertransport::X86Encoding(_) =>
                writeln!(f, "HyperTransport: X86 (reserved)"),
            v => writeln!(f, "HyperTransport: #{:02x}", u8::from(v)),

        }
    }
}



fn fmt_pri<'a>(f: &mut fmt::Formatter<'_>, sopi: &'a SlaveOrPrimaryInterface, verbose: usize) -> fmt::Result {
    writeln!(f, "HyperTransport: Slave or Primary Interface")?;
    if verbose < 2 {
        return Ok(());
    }
    let SlaveOrPrimaryInterface {
        command: cmd,
        link_control_0: lctr0,
        link_config_0: lcnf0,
        revision_id,
        link_control_1: lctr1,
        link_config_1: lcnf1,
        link_error_0,
        link_freq_cap_0: lfcap0,
        feature,
        link_error_1,
        link_freq_cap_1: lfcap1,
        error_handling: eh,
        ..
    } = sopi;
    let rid = u8::from(revision_id);
    if rid < 0x22 && rid > 0x11 {
        writeln!(f, "\t\t!!! Possibly incomplete decoding")?;
    }
    write!(f,
        "\t\tCommand: BaseUnitID={} UnitCnt={} MastHost{} DefDir{}",
        cmd.base_unitid,
        cmd.unit_count,
        cmd.master_host.display(BoolView::PlusMinus),
        cmd.default_direction.display(BoolView::PlusMinus),
    )?;
    if rid >= 0x22 {
        write!(f, " DUL{}", cmd.drop_on_uninitialized_link.display(BoolView::PlusMinus))?;
    }
    writeln!(f)?;
    write!(f,
        "\t\tLink Control 0: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        lctr0.crc_flood_enable.display(BoolView::PlusMinus),
        lctr0.crc_start_test.display(BoolView::PlusMinus),
        lctr0.crc_force_error.display(BoolView::PlusMinus),
        lctr0.link_failure.display(BoolView::PlusMinus),
        lctr0.initialization_complete.display(BoolView::PlusMinus),
        lctr0.end_of_chain.display(BoolView::PlusMinus),
        lctr0.transmitter_off.display(BoolView::PlusMinus),
        lctr0.crc_error,
    )?;
    if rid >= 0x22 {
        write!(f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            lctr0.isochronous_flow_control_enable.display(BoolView::PlusMinus),
            lctr0.ldtstop_tristate_enable.display(BoolView::PlusMinus),
            lctr0.extended_ctl_time.display(BoolView::PlusMinus),
            lctr0.enable_64_bit_addressing.display(BoolView::PlusMinus),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(f,
            "\t\tLink Config 0: MLWI={} MLWO={} LWI={} LWO={}",
            lcnf0.max_link_width_in.display(()),
            lcnf0.max_link_width_out.display(()),
            lcnf0.link_width_in.display(()),
            lcnf0.link_width_out.display(()),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config 0: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            lcnf0.max_link_width_in.display(()),
            lcnf0.doubleword_flow_control_in.display(BoolView::PlusMinus),
            lcnf0.max_link_width_out.display(()),
            lcnf0.doubleword_flow_control_out.display(BoolView::PlusMinus),
            lcnf0.link_width_in.display(()),
            lcnf0.doubleword_flow_control_in_enable.display(BoolView::PlusMinus),
            lcnf0.link_width_out.display(()),
            lcnf0.doubleword_flow_control_out_enable.display(BoolView::PlusMinus),
        )?;
    }
    write!(f,
        "\t\tLink Control 1: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        lctr1.crc_flood_enable.display(BoolView::PlusMinus),
        lctr1.crc_start_test.display(BoolView::PlusMinus),
        lctr1.crc_force_error.display(BoolView::PlusMinus),
        lctr1.link_failure.display(BoolView::PlusMinus),
        lctr1.initialization_complete.display(BoolView::PlusMinus),
        lctr1.end_of_chain.display(BoolView::PlusMinus),
        lctr1.transmitter_off.display(BoolView::PlusMinus),
        lctr1.crc_error,
    )?;
    if rid >= 0x22 {
        write!(f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            lctr1.isochronous_flow_control_enable.display(BoolView::PlusMinus),
            lctr1.ldtstop_tristate_enable.display(BoolView::PlusMinus),
            lctr1.extended_ctl_time.display(BoolView::PlusMinus),
            lctr1.enable_64_bit_addressing.display(BoolView::PlusMinus),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(f,
            "\t\tLink Config 1: MLWI={} MLWO={} LWI={} LWO={}",
            lcnf1.max_link_width_in.display(()),
            lcnf1.max_link_width_out.display(()),
            lcnf1.link_width_in.display(()),
            lcnf1.link_width_out.display(()),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config 1: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            lcnf1.max_link_width_in.display(()),
            lcnf1.doubleword_flow_control_in.display(BoolView::PlusMinus),
            lcnf1.max_link_width_out.display(()),
            lcnf1.doubleword_flow_control_out.display(BoolView::PlusMinus),
            lcnf1.link_width_in.display(()),
            lcnf1.doubleword_flow_control_in_enable.display(BoolView::PlusMinus),
            lcnf1.link_width_out.display(()),
            lcnf1.doubleword_flow_control_out_enable.display(BoolView::PlusMinus),
        )?;
    }
    writeln!(f, "\t\tRevision ID: {}.{:02}", revision_id.major, revision_id.minor)?;
    if rid < 0x22 {
        return Ok(());
    }
    writeln!(f, "\t\tLink Frequency 0: {}", sopi.link_freq_0(false).display(()))?;
    writeln!(f,
        "\t\tLink Error 0: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        link_error_0.protocol_error.display(BoolView::PlusMinus),
        link_error_0.overflow_error.display(BoolView::PlusMinus),
        link_error_0.end_of_chain_error.display(BoolView::PlusMinus),
        link_error_0.ctl_timeout.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tLink Frequency Capability 0: 200MHz{} 300MHz{} 400MHz{} 500MHz{} 600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        lfcap0.supports_200mhz.display(BoolView::PlusMinus),
        lfcap0.supports_300mhz.display(BoolView::PlusMinus),
        lfcap0.supports_400mhz.display(BoolView::PlusMinus),
        lfcap0.supports_500mhz.display(BoolView::PlusMinus),
        lfcap0.supports_600mhz.display(BoolView::PlusMinus),
        lfcap0.supports_800mhz.display(BoolView::PlusMinus),
        lfcap0.supports_1000mhz.display(BoolView::PlusMinus),
        lfcap0.supports_1200mhz.display(BoolView::PlusMinus),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // lfcap0.supports_1400mhz.display(BoolView::PlusMinus),
        // lfcap0.supports_1600mhz.display(BoolView::PlusMinus),
        // lfcap0.supports_vendor_specific.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tFeature Capability: IsocFC{} LDTSTOP{} CRCTM{} ECTLT{} 64bA{} UIDRD{}",
        feature.isochronous_flow_control_mode.display(BoolView::PlusMinus),
        feature.ldtstop.display(BoolView::PlusMinus),
        feature.crc_test_mode.display(BoolView::PlusMinus),
        feature.extended_ctl_time_required.display(BoolView::PlusMinus),
        feature.qword_addressing.display(BoolView::PlusMinus),
        feature.unitid_reorder_disable.display(BoolView::PlusMinus),
    )?;
    writeln!(f, "\t\tLink Frequency 1: {}", sopi.link_freq_1(false).display(()))?;
    writeln!(f,
        "\t\tLink Error 1: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        link_error_1.protocol_error.display(BoolView::PlusMinus),
        link_error_1.overflow_error.display(BoolView::PlusMinus),
        link_error_1.end_of_chain_error.display(BoolView::PlusMinus),
        link_error_1.ctl_timeout.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tLink Frequency Capability 1: 200MHz{} 300MHz{} 400MHz{} 500MHz{} 600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        lfcap1.supports_200mhz.display(BoolView::PlusMinus),
        lfcap1.supports_300mhz.display(BoolView::PlusMinus),
        lfcap1.supports_400mhz.display(BoolView::PlusMinus),
        lfcap1.supports_500mhz.display(BoolView::PlusMinus),
        lfcap1.supports_600mhz.display(BoolView::PlusMinus),
        lfcap1.supports_800mhz.display(BoolView::PlusMinus),
        lfcap1.supports_1000mhz.display(BoolView::PlusMinus),
        lfcap1.supports_1200mhz.display(BoolView::PlusMinus),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // lfcap1.supports_1400mhz.display(BoolView::PlusMinus),
        // lfcap1.supports_1600mhz.display(BoolView::PlusMinus),
        // lfcap1.supports_vendor_specific.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tError Handling: PFlE{} OFlE{} PFE{} OFE{} EOCFE{} RFE{} CRCFE{} SERRFE{} CF{} RE{} PNFE{} ONFE{} EOCNFE{} RNFE{} CRCNFE{} SERRNFE{}",
        eh.protocol_error_flood_enable.display(BoolView::PlusMinus),
        eh.overflow_error_flood_enable.display(BoolView::PlusMinus),
        eh.protocol_error_fatal_enable.display(BoolView::PlusMinus),
        eh.overflow_error_fatal_enable.display(BoolView::PlusMinus),
        eh.end_of_chain_error_fatal_enable.display(BoolView::PlusMinus),
        eh.response_error_fatal_enable.display(BoolView::PlusMinus),
        eh.crc_error_fatal_enable.display(BoolView::PlusMinus),
        eh.system_error_fatal_enable.display(BoolView::PlusMinus),
        eh.chain_fail.display(BoolView::PlusMinus),
        eh.response_error.display(BoolView::PlusMinus),
        eh.protocol_error_nonfatal_enable.display(BoolView::PlusMinus),
        eh.overflow_error_nonfatal_enable.display(BoolView::PlusMinus),
        eh.end_of_chain_error_nonfatal_enable.display(BoolView::PlusMinus),
        eh.response_error_nonfatal_enable.display(BoolView::PlusMinus),
        eh.crc_error_nonfatal_enable.display(BoolView::PlusMinus),
        eh.system_error_nonfatal_enable.display(BoolView::PlusMinus),
    )?;
    writeln!(f, 
        "\t\tPrefetchable memory behind bridge Upper: {:02x}-{:02x}",
        sopi.mem_base_upper,
        sopi.mem_limit_upper
    )?;
    writeln!(f, "\t\tBus Number: {:02x}", sopi.bus_number)
}

fn fmt_sec<'a>(f: &mut fmt::Formatter<'_>, hosi: &'a HostOrSecondaryInterface, verbose: usize) -> fmt::Result {
    writeln!(f, "HyperTransport: Host or Secondary Interface")?;
    if verbose < 2 {
        return Ok(());
    }
    let HostOrSecondaryInterface {
        command,
        link_control,
        link_config,
        revision_id,
        link_error,
        link_freq_cap,
        feature,
        error_handling,
        ..
    } = hosi;
    let rid = u8::from(revision_id);
    if rid < 0x22 && rid > 0x11 {
        writeln!(f, "\t\t!!! Possibly incomplete decoding")?;
    }
    write!(f,
        "\t\tCommand: WarmRst{} DblEnd{}",
        command.warm_reset.display(BoolView::PlusMinus),
        command.double_ended.display(BoolView::PlusMinus),
    )?;
    if rid >= 0x22 {
        writeln!(f,
            " DevNum={} ChainSide{} HostHide{} Slave{} <EOCErr{} DUL{}",
            command.device_number,
            command.chain_side.display(BoolView::PlusMinus),
            command.host_hide.display(BoolView::PlusMinus),
            command.act_as_slave.display(BoolView::PlusMinus),
            command.host_inbound_end_of_chain_error.display(BoolView::PlusMinus),
            command.drop_on_uninitialized_link.display(BoolView::PlusMinus),
        )?;
    }
    write!(f,
        "\t\tLink Control: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        link_control.crc_flood_enable.display(BoolView::PlusMinus),
        link_control.crc_start_test.display(BoolView::PlusMinus),
        link_control.crc_force_error.display(BoolView::PlusMinus),
        link_control.link_failure.display(BoolView::PlusMinus),
        link_control.initialization_complete.display(BoolView::PlusMinus),
        link_control.end_of_chain.display(BoolView::PlusMinus),
        link_control.transmitter_off.display(BoolView::PlusMinus),
        link_control.crc_error,
    )?;
    if rid >= 0x22 {
        write!(f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            link_control.isochronous_flow_control_enable.display(BoolView::PlusMinus),
            link_control.ldtstop_tristate_enable.display(BoolView::PlusMinus),
            link_control.extended_ctl_time.display(BoolView::PlusMinus),
            link_control.enable_64_bit_addressing.display(BoolView::PlusMinus),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(f,
            "\t\tLink Config: MLWI={} MLWO={} LWI={} LWO={}",
            link_config.max_link_width_in.display(()),
            link_config.max_link_width_out.display(()),
            link_config.link_width_in.display(()),
            link_config.link_width_out.display(()),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            link_config.max_link_width_in.display(()),
            link_config.doubleword_flow_control_in.display(BoolView::PlusMinus),
            link_config.max_link_width_out.display(()),
            link_config.doubleword_flow_control_out.display(BoolView::PlusMinus),
            link_config.link_width_in.display(()),
            link_config.doubleword_flow_control_in_enable.display(BoolView::PlusMinus),
            link_config.link_width_out.display(()),
            link_config.doubleword_flow_control_out_enable.display(BoolView::PlusMinus),
        )?;
    }
    writeln!(f, "\t\tRevision ID: {}.{:02}", revision_id.major, revision_id.minor)?;
    if rid < 0x22 {
        return Ok(());
    }
    writeln!(f, "\t\tLink Frequency: {}", hosi.link_freq(false).display(()))?;
    writeln!(f,
        "\t\tLink Error: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        link_error.protocol_error.display(BoolView::PlusMinus),
        link_error.overflow_error.display(BoolView::PlusMinus),
        link_error.end_of_chain_error.display(BoolView::PlusMinus),
        link_error.ctl_timeout.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tLink Frequency Capability: 200MHz{} 300MHz{} 400MHz{} 500MHz{} \
        600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        link_freq_cap.supports_200mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_300mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_400mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_500mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_600mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_800mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_1000mhz.display(BoolView::PlusMinus),
        link_freq_cap.supports_1200mhz.display(BoolView::PlusMinus),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // lfcap0.supports_1400mhz.display(BoolView::PlusMinus),
        // lfcap0.supports_1600mhz.display(BoolView::PlusMinus),
        // lfcap0.supports_vendor_specific.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
        false.display(BoolView::PlusMinus),
    )?;
    writeln!(f,
        "\t\tFeature Capability: IsocFC{} LDTSTOP{} CRCTM{} ECTLT{} 64bA{} UIDRD{} ExtRS{} UCnfE{}",
        feature.isochronous_flow_control_mode.display(BoolView::PlusMinus),
        feature.ldtstop.display(BoolView::PlusMinus),
        feature.crc_test_mode.display(BoolView::PlusMinus),
        feature.extended_ctl_time_required.display(BoolView::PlusMinus),
        feature.qword_addressing.display(BoolView::PlusMinus),
        feature.unitid_reorder_disable.display(BoolView::PlusMinus),
        feature.extended_register_set.display(BoolView::PlusMinus),
        feature.upstream_configuration_enable.display(BoolView::PlusMinus),
    )?;
    if feature.extended_register_set {
        writeln!(f,
            "\t\tError Handling: PFlE{} OFlE{} PFE{} OFE{} EOCFE{} RFE{} CRCFE{} SERRFE{} \
            CF{} RE{} PNFE{} ONFE{} EOCNFE{} RNFE{} CRCNFE{} SERRNFE{}",
            error_handling.protocol_error_flood_enable.display(BoolView::PlusMinus),
            error_handling.overflow_error_flood_enable.display(BoolView::PlusMinus),
            error_handling.protocol_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.overflow_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.end_of_chain_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.response_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.crc_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.system_error_fatal_enable.display(BoolView::PlusMinus),
            error_handling.chain_fail.display(BoolView::PlusMinus),
            error_handling.response_error.display(BoolView::PlusMinus),
            error_handling.protocol_error_nonfatal_enable.display(BoolView::PlusMinus),
            error_handling.overflow_error_nonfatal_enable.display(BoolView::PlusMinus),
            error_handling.end_of_chain_error_nonfatal_enable.display(BoolView::PlusMinus),
            error_handling.response_error_nonfatal_enable.display(BoolView::PlusMinus),
            error_handling.crc_error_nonfatal_enable.display(BoolView::PlusMinus),
            error_handling.system_error_nonfatal_enable.display(BoolView::PlusMinus),
        )?;
        writeln!(f, 
            "\t\tPrefetchable memory behind bridge Upper: {:02x}-{:02x}",
            hosi.mem_base_upper,
            hosi.mem_limit_upper
        )?;
    }
    Ok(())
}

fn fmt_msim<'a>(f: &mut fmt::Formatter<'_>, msim: &'a MsiMapping, verbose: usize) -> fmt::Result {
    writeln!(f,
        "HyperTransport: MSI Mapping Enable{} Fixed{}",
        msim.enabled.display(BoolView::PlusMinus),
        msim.fixed.display(BoolView::PlusMinus),
    )?;
    if verbose >= 2 && !msim.fixed {
        writeln!(f, "\t\tMapping Address Base: {:016x}", msim.base_address())?;
    }
    Ok(())
}

impl<'a> DisplayMultiViewBasic<()> for LinkWidth {}
impl<'a> fmt::Display for MultiView<&'a LinkWidth, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            LinkWidth::Width8bits => "8bit",
            LinkWidth::Width16bits => "16bit",
            LinkWidth::Reserved(2) => "[2]",
            LinkWidth::Width32bits => "32bit",
            LinkWidth::Width2bits => "2bit",
            LinkWidth::Width4bits => "4bit",
            LinkWidth::Reserved(6) => "[6]",
            LinkWidth::NotConnected => "N/C",
            _ => unreachable!(),
        };
        write!(f, "{}", s)
    }
}

impl<'a> DisplayMultiViewBasic<()> for LinkFrequency {}
impl<'a> fmt::Display for MultiView<&'a LinkFrequency, ()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.data {
            LinkFrequency::Rate200MHz     => "200MHz",
            LinkFrequency::Rate300MHz     => "300MHz",
            LinkFrequency::Rate400MHz     => "400MHz",
            LinkFrequency::Rate500MHz     => "500MHz",
            LinkFrequency::Rate600MHz     => "600MHz",
            LinkFrequency::Rate800MHz     => "800MHz",
            LinkFrequency::Rate1000MHz    => "1.0GHz",
            LinkFrequency::Rate1200MHz    => "1.2GHz",
            LinkFrequency::Rate1400MHz    => "1.4GHz",
            LinkFrequency::Rate1600MHz    => "1.6GHz",
            LinkFrequency::Rate1800MHz    => "[a]",
            LinkFrequency::Rate2000MHz    => "[b]",
            LinkFrequency::Rate2200MHz    => "[c]",
            LinkFrequency::Rate2400MHz    => "[d]",
            LinkFrequency::Rate2600MHz    => "[e]",
            LinkFrequency::VendorSpecific => "Vend",
            LinkFrequency::Rate2800MHz    => "2.8GHz",
            LinkFrequency::Rate3000MHz    => "3.0GHz",
            LinkFrequency::Rate3200MHz    => "3.2GHz",
            LinkFrequency::Reserved(_)   => "Rsvd",
        };
        write!(f, "{}", s)
    }
}
