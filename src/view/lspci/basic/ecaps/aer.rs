use core::fmt;

use pcics::extended_capabilities::AdvancedErrorReporting;

use super::{Flag, View};

pub(super) struct ViewArgs {
    pub(super) verbose: usize,
    pub(super) is_type_root: bool,
}

impl<'a> fmt::Display for View<&'a AdvancedErrorReporting, &'a ViewArgs> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AdvancedErrorReporting {
            uncorrectable_error_status: ue_st,
            uncorrectable_error_mask: ue_msk,
            uncorrectable_error_severity: ue_svrt,
            correctable_error_status: ce_st,
            correctable_error_mask: ce_msk,
            advanced_error_capabilities_and_control: aer_caps,
            header_log: hl,
            root_error_command: root_cmd,
            root_error_status: root_st,
            error_source_identification: esi,
            ..
        } = self.data;
        let &ViewArgs {
            verbose,
            is_type_root,
        } = self.args;
        writeln!(f, "Advanced Error Reporting")?;
        if verbose < 2 {
            return Ok(());
        }
        write!(
            f,
            "\t\tUESta:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            Flag(ue_st.data_link_protocol_error_status),
            Flag(ue_st.surprise_down_error_status),
            Flag(ue_st.poisoned_tlp_received_status),
            Flag(ue_st.flow_control_protocol_error_status),
            Flag(ue_st.completion_timeout_status),
            Flag(ue_st.completer_abort_status),
            Flag(ue_st.unexpected_completion_status),
            Flag(ue_st.receiver_overflow_status),
        )?;
        writeln!(
            f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            Flag(ue_st.malformed_tlp_status),
            Flag(ue_st.ecrc_error_status),
            Flag(ue_st.unsupported_request_error_status),
            Flag(ue_st.acs_violation_status),
        )?;
        write!(
            f,
            "\t\tUEMsk:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            Flag(ue_msk.data_link_protocol_error_status),
            Flag(ue_msk.surprise_down_error_status),
            Flag(ue_msk.poisoned_tlp_received_status),
            Flag(ue_msk.flow_control_protocol_error_status),
            Flag(ue_msk.completion_timeout_status),
            Flag(ue_msk.completer_abort_status),
            Flag(ue_msk.unexpected_completion_status),
            Flag(ue_msk.receiver_overflow_status),
        )?;
        writeln!(
            f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            Flag(ue_msk.malformed_tlp_status),
            Flag(ue_msk.ecrc_error_status),
            Flag(ue_msk.unsupported_request_error_status),
            Flag(ue_msk.acs_violation_status),
        )?;
        write!(
            f,
            "\t\tUESvrt:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            Flag(ue_svrt.data_link_protocol_error_status),
            Flag(ue_svrt.surprise_down_error_status),
            Flag(ue_svrt.poisoned_tlp_received_status),
            Flag(ue_svrt.flow_control_protocol_error_status),
            Flag(ue_svrt.completion_timeout_status),
            Flag(ue_svrt.completer_abort_status),
            Flag(ue_svrt.unexpected_completion_status),
            Flag(ue_svrt.receiver_overflow_status),
        )?;
        writeln!(
            f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            Flag(ue_svrt.malformed_tlp_status),
            Flag(ue_svrt.ecrc_error_status),
            Flag(ue_svrt.unsupported_request_error_status),
            Flag(ue_svrt.acs_violation_status),
        )?;
        writeln!(
            f,
            "\t\tCESta:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}",
            Flag(ce_st.receiver_error_status),
            Flag(ce_st.bad_tlp_status),
            Flag(ce_st.bad_dllp_status),
            Flag(ce_st.replay_num_rollover_status),
            Flag(ce_st.replay_timer_timeout_status),
            Flag(ce_st.advisory_non_fatal_error_status),
        )?;
        writeln!(
            f,
            "\t\tCEMsk:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}",
            Flag(ce_msk.receiver_error_status),
            Flag(ce_msk.bad_tlp_status),
            Flag(ce_msk.bad_dllp_status),
            Flag(ce_msk.replay_num_rollover_status),
            Flag(ce_msk.replay_timer_timeout_status),
            Flag(ce_msk.advisory_non_fatal_error_status),
        )?;
        writeln!(f,
            "\t\tAERCap:\tFirst Error Pointer: {:02x}, ECRCGenCap{} ECRCGenEn{} ECRCChkCap{} ECRCChkEn{}",
            aer_caps.first_error_pointer,
            Flag(aer_caps.ecrc_generation_capable),
            Flag(aer_caps.ecrc_generation_enable),
            Flag(aer_caps.ecrc_check_capable),
            Flag(aer_caps.ecrc_check_enable),
        )?;
        writeln!(
            f,
            "\t\t\tMultHdrRecCap{} MultHdrRecEn{} TLPPfxPres{} HdrLogCap{}",
            Flag(aer_caps.multiple_header_recording_capable),
            Flag(aer_caps.multiple_header_recording_enable),
            Flag(aer_caps.tlp_prefix_log_present),
            Flag(aer_caps.completion_timeout_prefix_or_header_log_capable),
        )?;
        writeln!(
            f,
            "\t\tHeaderLog: {:08x} {:08x} {:08x} {:08x}",
            hl.0[0], hl.0[1], hl.0[2], hl.0[3]
        )?;
        if let (true, Some(root_cmd), Some(root_st), Some(esi)) =
            (is_type_root, root_cmd, root_st, esi)
        {
            writeln!(
                f,
                "\t\tRootCmd: CERptEn{} NFERptEn{} FERptEn{}",
                Flag(root_cmd.correctable_error_reporting_enable),
                Flag(root_cmd.non_fatal_error_reporting_enable),
                Flag(root_cmd.fatal_error_reporting_enable),
            )?;
            writeln!(
                f,
                "\t\tRootSta: CERcvd{} MultCERcvd{} UERcvd{} MultUERcvd{}",
                Flag(root_st.err_cor_received),
                Flag(root_st.multiple_err_cor_received),
                Flag(root_st.err_fatal_or_nonfatal_received),
                Flag(root_st.multiple_err_fatal_or_nonfatal_received),
            )?;
            writeln!(
                f,
                "\t\t\t FirstFatal{} NonFatalMsg{} FatalMsg{} IntMsg {}",
                Flag(root_st.first_uncorrectable_fatal),
                Flag(root_st.non_fatal_error_messages_received),
                Flag(root_st.fatal_error_messages_received),
                root_st.advanced_error_interrupt_message_number,
            )?;
            write!(
                f,
                "\t\tErrorSrc: ERR_COR: {:04x} ",
                esi.err_cor_source_identification
            )?;
            writeln!(
                f,
                "ERR_FATAL/NONFATAL: {:04x}",
                esi.err_fatal_or_nonfatal_source_identification
            )?;
        }
        Ok(())
    }
}
