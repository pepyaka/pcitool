use core::fmt;

use pcics::capabilities::hypertransport::{
    HostOrSecondaryInterface, Hypertransport, LinkFrequency, LinkWidth, MsiMapping,
    SlaveOrPrimaryInterface,
};

use super::{Flag, Simple, Verbose};

impl<'a> fmt::Display for Verbose<&'a Hypertransport> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verbose = self.verbose;
        match self.data {
            Hypertransport::SlaveOrPrimaryInterface(sopi) => fmt_pri(f, sopi, verbose),
            Hypertransport::HostOrSecondaryInterface(hosi) => fmt_sec(f, hosi, verbose),
            Hypertransport::Switch(_) => writeln!(f, "HyperTransport: Switch"),
            Hypertransport::InterruptDiscoveryAndConfiguration(_) => {
                writeln!(f, "HyperTransport: Interrupt Discovery and Configuration")
            }
            Hypertransport::RevisionId(id) => writeln!(
                f,
                "HyperTransport: Revision ID: {}.{:02}",
                id.major, id.minor
            ),
            Hypertransport::UnitIdClumping(_) => writeln!(f, "HyperTransport: UnitID Clumping"),
            Hypertransport::ExtendedConfigurationSpaceAccess(_) => {
                writeln!(f, "HyperTransport: Extended Configuration Space Access")
            }
            Hypertransport::AddressMapping(_) => writeln!(f, "HyperTransport: Address Mapping"),
            Hypertransport::MsiMapping(msim) => fmt_msim(f, msim, verbose),
            Hypertransport::DirectRoute(_) => writeln!(f, "HyperTransport: DirectRoute"),
            Hypertransport::VCSet(_) => writeln!(f, "HyperTransport: VCSet"),
            Hypertransport::RetryMode(_) => writeln!(f, "HyperTransport: Retry Mode"),
            Hypertransport::X86Encoding(_) => writeln!(f, "HyperTransport: X86 (reserved)"),
            v => writeln!(f, "HyperTransport: #{:02x}", u8::from(v)),
        }
    }
}

fn fmt_pri<'a>(
    f: &mut fmt::Formatter<'_>,
    sopi: &'a SlaveOrPrimaryInterface,
    verbose: usize,
) -> fmt::Result {
    writeln!(f, "HyperTransport: Slave or Primary Interface")?;
    if verbose < 2 {
        return Ok(());
    }
    let SlaveOrPrimaryInterface {
        command: cmd,
        link_control_0: lctr0,
        link_config_0,
        revision_id,
        link_control_1: lctr1,
        link_config_1,
        link_error_0,
        link_freq_cap_0: lfcap0,
        feature,
        link_error_1,
        link_freq_cap_1: lfcap1,
        error_handling: eh,
        ..
    } = sopi;
    let lcnf0 = link_config_0.clone();
    let lcnf1 = link_config_1.clone();
    let rid = u8::from(revision_id);
    if rid < 0x22 && rid > 0x11 {
        writeln!(f, "\t\t!!! Possibly incomplete decoding")?;
    }
    write!(
        f,
        "\t\tCommand: BaseUnitID={} UnitCnt={} MastHost{} DefDir{}",
        cmd.base_unitid,
        cmd.unit_count,
        Flag(cmd.master_host),
        Flag(cmd.default_direction),
    )?;
    if rid >= 0x22 {
        write!(f, " DUL{}", Flag(cmd.drop_on_uninitialized_link))?;
    }
    writeln!(f)?;
    write!(
        f,
        "\t\tLink Control 0: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        Flag(lctr0.crc_flood_enable),
        Flag(lctr0.crc_start_test),
        Flag(lctr0.crc_force_error),
        Flag(lctr0.link_failure),
        Flag(lctr0.initialization_complete),
        Flag(lctr0.end_of_chain),
        Flag(lctr0.transmitter_off),
        lctr0.crc_error,
    )?;
    if rid >= 0x22 {
        write!(
            f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            Flag(lctr0.isochronous_flow_control_enable),
            Flag(lctr0.ldtstop_tristate_enable),
            Flag(lctr0.extended_ctl_time),
            Flag(lctr0.enable_64_bit_addressing),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(
            f,
            "\t\tLink Config 0: MLWI={} MLWO={} LWI={} LWO={}",
            Simple(lcnf0.max_link_width_in),
            Simple(lcnf0.max_link_width_out),
            Simple(lcnf0.link_width_in),
            Simple(lcnf0.link_width_out),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config 0: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            Simple(lcnf0.max_link_width_in),
            Flag(lcnf0.doubleword_flow_control_in),
            Simple(lcnf0.max_link_width_out),
            Flag(lcnf0.doubleword_flow_control_out),
            Simple(lcnf0.link_width_in),
            Flag(lcnf0.doubleword_flow_control_in_enable),
            Simple(lcnf0.link_width_out),
            Flag(lcnf0.doubleword_flow_control_out_enable),
        )?;
    }
    write!(
        f,
        "\t\tLink Control 1: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        Flag(lctr1.crc_flood_enable),
        Flag(lctr1.crc_start_test),
        Flag(lctr1.crc_force_error),
        Flag(lctr1.link_failure),
        Flag(lctr1.initialization_complete),
        Flag(lctr1.end_of_chain),
        Flag(lctr1.transmitter_off),
        lctr1.crc_error,
    )?;
    if rid >= 0x22 {
        write!(
            f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            Flag(lctr1.isochronous_flow_control_enable),
            Flag(lctr1.ldtstop_tristate_enable),
            Flag(lctr1.extended_ctl_time),
            Flag(lctr1.enable_64_bit_addressing),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(
            f,
            "\t\tLink Config 1: MLWI={} MLWO={} LWI={} LWO={}",
            Simple(lcnf1.max_link_width_in),
            Simple(lcnf1.max_link_width_out),
            Simple(lcnf1.link_width_in),
            Simple(lcnf1.link_width_out),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config 1: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            Simple(lcnf1.max_link_width_in),
            Flag(lcnf1.doubleword_flow_control_in),
            Simple(lcnf1.max_link_width_out),
            Flag(lcnf1.doubleword_flow_control_out),
            Simple(lcnf1.link_width_in),
            Flag(lcnf1.doubleword_flow_control_in_enable),
            Simple(lcnf1.link_width_out),
            Flag(lcnf1.doubleword_flow_control_out_enable),
        )?;
    }
    writeln!(
        f,
        "\t\tRevision ID: {}.{:02}",
        revision_id.major, revision_id.minor
    )?;
    if rid < 0x22 {
        return Ok(());
    }
    writeln!(
        f,
        "\t\tLink Frequency 0: {}",
        Simple(sopi.link_freq_0(false))
    )?;
    writeln!(
        f,
        "\t\tLink Error 0: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        Flag(link_error_0.protocol_error),
        Flag(link_error_0.overflow_error),
        Flag(link_error_0.end_of_chain_error),
        Flag(link_error_0.ctl_timeout),
    )?;
    writeln!(f,
        "\t\tLink Frequency Capability 0: 200MHz{} 300MHz{} 400MHz{} 500MHz{} 600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        Flag(lfcap0.supports_200mhz),
        Flag(lfcap0.supports_300mhz),
        Flag(lfcap0.supports_400mhz),
        Flag(lfcap0.supports_500mhz),
        Flag(lfcap0.supports_600mhz),
        Flag(lfcap0.supports_800mhz),
        Flag(lfcap0.supports_1000mhz),
        Flag(lfcap0.supports_1200mhz),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // Flag(lfcap0.supports_1400mhz),
        // Flag(lfcap0.supports_1600mhz),
        // Flag(lfcap0.supports_vendor_specific),
        Flag(false),
        Flag(false),
        Flag(false),
    )?;
    writeln!(
        f,
        "\t\tFeature Capability: IsocFC{} LDTSTOP{} CRCTM{} ECTLT{} 64bA{} UIDRD{}",
        Flag(feature.isochronous_flow_control_mode),
        Flag(feature.ldtstop),
        Flag(feature.crc_test_mode),
        Flag(feature.extended_ctl_time_required),
        Flag(feature.qword_addressing),
        Flag(feature.unitid_reorder_disable),
    )?;
    writeln!(
        f,
        "\t\tLink Frequency 1: {}",
        Simple(sopi.link_freq_1(false))
    )?;
    writeln!(
        f,
        "\t\tLink Error 1: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        Flag(link_error_1.protocol_error),
        Flag(link_error_1.overflow_error),
        Flag(link_error_1.end_of_chain_error),
        Flag(link_error_1.ctl_timeout),
    )?;
    writeln!(f,
        "\t\tLink Frequency Capability 1: 200MHz{} 300MHz{} 400MHz{} 500MHz{} 600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        Flag(lfcap1.supports_200mhz),
        Flag(lfcap1.supports_300mhz),
        Flag(lfcap1.supports_400mhz),
        Flag(lfcap1.supports_500mhz),
        Flag(lfcap1.supports_600mhz),
        Flag(lfcap1.supports_800mhz),
        Flag(lfcap1.supports_1000mhz),
        Flag(lfcap1.supports_1200mhz),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // Flag(lfcap1.supports_1400mhz),
        // Flag(lfcap1.supports_1600mhz),
        // Flag(lfcap1.supports_vendor_specific),
        Flag(false),
        Flag(false),
        Flag(false),
    )?;
    writeln!(f,
        "\t\tError Handling: PFlE{} OFlE{} PFE{} OFE{} EOCFE{} RFE{} CRCFE{} SERRFE{} CF{} RE{} PNFE{} ONFE{} EOCNFE{} RNFE{} CRCNFE{} SERRNFE{}",
        Flag(eh.protocol_error_flood_enable),
        Flag(eh.overflow_error_flood_enable),
        Flag(eh.protocol_error_fatal_enable),
        Flag(eh.overflow_error_fatal_enable),
        Flag(eh.end_of_chain_error_fatal_enable),
        Flag(eh.response_error_fatal_enable),
        Flag(eh.crc_error_fatal_enable),
        Flag(eh.system_error_fatal_enable),
        Flag(eh.chain_fail),
        Flag(eh.response_error),
        Flag(eh.protocol_error_nonfatal_enable),
        Flag(eh.overflow_error_nonfatal_enable),
        Flag(eh.end_of_chain_error_nonfatal_enable),
        Flag(eh.response_error_nonfatal_enable),
        Flag(eh.crc_error_nonfatal_enable),
        Flag(eh.system_error_nonfatal_enable),
    )?;
    writeln!(
        f,
        "\t\tPrefetchable memory behind bridge Upper: {:02x}-{:02x}",
        sopi.mem_base_upper, sopi.mem_limit_upper
    )?;
    writeln!(f, "\t\tBus Number: {:02x}", sopi.bus_number)
}

fn fmt_sec<'a>(
    f: &mut fmt::Formatter<'_>,
    hosi: &'a HostOrSecondaryInterface,
    verbose: usize,
) -> fmt::Result {
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
    let link_config = link_config.clone();
    let rid = u8::from(revision_id);
    if rid < 0x22 && rid > 0x11 {
        writeln!(f, "\t\t!!! Possibly incomplete decoding")?;
    }
    write!(
        f,
        "\t\tCommand: WarmRst{} DblEnd{}",
        Flag(command.warm_reset),
        Flag(command.double_ended),
    )?;
    if rid >= 0x22 {
        writeln!(
            f,
            " DevNum={} ChainSide{} HostHide{} Slave{} <EOCErr{} DUL{}",
            // There is a bug in pciutils: PCI_HT_SEC_CMD_DN = 0x0076, should be 0x007C
            command.device_number & 0b11101,
            Flag(command.chain_side),
            Flag(command.host_hide),
            Flag(command.act_as_slave),
            Flag(command.host_inbound_end_of_chain_error),
            Flag(command.drop_on_uninitialized_link),
        )?;
    }
    write!(
        f,
        "\t\tLink Control: CFlE{} CST{} CFE{} <LkFail{} Init{} EOC{} TXO{} <CRCErr={:x}",
        Flag(link_control.crc_flood_enable),
        Flag(link_control.crc_start_test),
        Flag(link_control.crc_force_error),
        Flag(link_control.link_failure),
        Flag(link_control.initialization_complete),
        Flag(link_control.end_of_chain),
        Flag(link_control.transmitter_off),
        link_control.crc_error,
    )?;
    if rid >= 0x22 {
        write!(
            f,
            " IsocEn{} LSEn{} ExtCTL{} 64b{}",
            Flag(link_control.isochronous_flow_control_enable),
            Flag(link_control.ldtstop_tristate_enable),
            Flag(link_control.extended_ctl_time),
            Flag(link_control.enable_64_bit_addressing),
        )?;
    }
    writeln!(f)?;
    if rid < 0x22 {
        writeln!(
            f,
            "\t\tLink Config: MLWI={} MLWO={} LWI={} LWO={}",
            Simple(link_config.max_link_width_in),
            Simple(link_config.max_link_width_out),
            Simple(link_config.link_width_in),
            Simple(link_config.link_width_out),
        )?;
    } else {
        writeln!(f,
            "\t\tLink Config: MLWI={} DwFcIn{} MLWO={} DwFcOut{} LWI={} DwFcInEn{} LWO={} DwFcOutEn{}",
            Simple(link_config.max_link_width_in),
            Flag(link_config.doubleword_flow_control_in),
            Simple(link_config.max_link_width_out),
            Flag(link_config.doubleword_flow_control_out),
            Simple(link_config.link_width_in),
            Flag(link_config.doubleword_flow_control_in_enable),
            Simple(link_config.link_width_out),
            Flag(link_config.doubleword_flow_control_out_enable),
        )?;
    }
    writeln!(
        f,
        "\t\tRevision ID: {}.{:02}",
        revision_id.major, revision_id.minor
    )?;
    if rid < 0x22 {
        return Ok(());
    }
    writeln!(f, "\t\tLink Frequency: {}", Simple(hosi.link_freq(false)))?;
    writeln!(
        f,
        "\t\tLink Error: <Prot{} <Ovfl{} <EOC{} CTLTm{}",
        Flag(link_error.protocol_error),
        Flag(link_error.overflow_error),
        Flag(link_error.end_of_chain_error),
        Flag(link_error.ctl_timeout),
    )?;
    writeln!(
        f,
        "\t\tLink Frequency Capability: 200MHz{} 300MHz{} 400MHz{} 500MHz{} \
        600MHz{} 800MHz{} 1.0GHz{} 1.2GHz{} 1.4GHz{} 1.6GHz{} Vend{}",
        Flag(link_freq_cap.supports_200mhz),
        Flag(link_freq_cap.supports_300mhz),
        Flag(link_freq_cap.supports_400mhz),
        Flag(link_freq_cap.supports_500mhz),
        Flag(link_freq_cap.supports_600mhz),
        Flag(link_freq_cap.supports_800mhz),
        Flag(link_freq_cap.supports_1000mhz),
        Flag(link_freq_cap.supports_1200mhz),
        // There is an error in ls-caps.c it get only u8, so vaues below always false
        // Flag(lfcap0.supports_1400mhz),
        // Flag(lfcap0.supports_1600mhz),
        // Flag(lfcap0.supports_vendor_specific),
        Flag(false),
        Flag(false),
        Flag(false),
    )?;
    writeln!(
        f,
        "\t\tFeature Capability: IsocFC{} LDTSTOP{} CRCTM{} ECTLT{} 64bA{} UIDRD{} ExtRS{} UCnfE{}",
        Flag(feature.isochronous_flow_control_mode),
        Flag(feature.ldtstop),
        Flag(feature.crc_test_mode),
        Flag(feature.extended_ctl_time_required),
        Flag(feature.qword_addressing),
        Flag(feature.unitid_reorder_disable),
        Flag(feature.extended_register_set),
        Flag(feature.upstream_configuration_enable),
    )?;
    if feature.extended_register_set {
        writeln!(
            f,
            "\t\tError Handling: PFlE{} OFlE{} PFE{} OFE{} EOCFE{} RFE{} CRCFE{} SERRFE{} \
            CF{} RE{} PNFE{} ONFE{} EOCNFE{} RNFE{} CRCNFE{} SERRNFE{}",
            Flag(error_handling.protocol_error_flood_enable),
            Flag(error_handling.overflow_error_flood_enable),
            Flag(error_handling.protocol_error_fatal_enable),
            Flag(error_handling.overflow_error_fatal_enable),
            Flag(error_handling.end_of_chain_error_fatal_enable),
            Flag(error_handling.response_error_fatal_enable),
            Flag(error_handling.crc_error_fatal_enable),
            Flag(error_handling.system_error_fatal_enable),
            Flag(error_handling.chain_fail),
            Flag(error_handling.response_error),
            Flag(error_handling.protocol_error_nonfatal_enable),
            Flag(error_handling.overflow_error_nonfatal_enable),
            Flag(error_handling.end_of_chain_error_nonfatal_enable),
            Flag(error_handling.response_error_nonfatal_enable),
            Flag(error_handling.crc_error_nonfatal_enable),
            Flag(error_handling.system_error_nonfatal_enable),
        )?;
        writeln!(
            f,
            "\t\tPrefetchable memory behind bridge Upper: {:02x}-{:02x}",
            hosi.mem_base_upper, hosi.mem_limit_upper
        )?;
    }
    Ok(())
}

fn fmt_msim<'a>(f: &mut fmt::Formatter<'_>, msim: &'a MsiMapping, verbose: usize) -> fmt::Result {
    writeln!(
        f,
        "HyperTransport: MSI Mapping Enable{} Fixed{}",
        Flag(msim.enabled),
        Flag(msim.fixed),
    )?;
    if verbose >= 2 && !msim.fixed {
        writeln!(f, "\t\tMapping Address Base: {:016x}", msim.base_address())?;
    }
    Ok(())
}

impl fmt::Display for Simple<LinkWidth> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
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

impl fmt::Display for Simple<LinkFrequency> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
            LinkFrequency::Rate200MHz => "200MHz",
            LinkFrequency::Rate300MHz => "300MHz",
            LinkFrequency::Rate400MHz => "400MHz",
            LinkFrequency::Rate500MHz => "500MHz",
            LinkFrequency::Rate600MHz => "600MHz",
            LinkFrequency::Rate800MHz => "800MHz",
            LinkFrequency::Rate1000MHz => "1.0GHz",
            LinkFrequency::Rate1200MHz => "1.2GHz",
            LinkFrequency::Rate1400MHz => "1.4GHz",
            LinkFrequency::Rate1600MHz => "1.6GHz",
            LinkFrequency::Rate1800MHz => "[a]",
            LinkFrequency::Rate2000MHz => "[b]",
            LinkFrequency::Rate2200MHz => "[c]",
            LinkFrequency::Rate2400MHz => "[d]",
            LinkFrequency::Rate2600MHz => "[e]",
            LinkFrequency::VendorSpecific => "Vend",
            LinkFrequency::Rate2800MHz => "2.8GHz",
            LinkFrequency::Rate3000MHz => "3.0GHz",
            LinkFrequency::Rate3200MHz => "3.2GHz",
            LinkFrequency::Reserved(_) => "Rsvd",
        };
        write!(f, "{}", s)
    }
}
