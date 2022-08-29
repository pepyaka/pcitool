use core::fmt;

use pcics::extended_capabilities::AdvancedErrorReporting;
use crate::{
    view::{
        DisplayMultiView,
        MultiView,
        BoolView
    },
};



pub struct AerView {
    pub verbose: usize,
    pub is_type_root: bool,
}


impl<'a> DisplayMultiView<AerView> for AdvancedErrorReporting {}
impl<'a> fmt::Display for MultiView<&'a AdvancedErrorReporting, AerView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AerView { verbose, is_type_root } = self.view;
        writeln!(f, "Advanced Error Reporting")?;
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
        if verbose < 2 {
            return Ok(());
        }
        write!(f,
            "\t\tUESta:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            ue_st.data_link_protocol_error_status.display(BoolView::PlusMinus),
            ue_st.surprise_down_error_status.display(BoolView::PlusMinus),
            ue_st.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            ue_st.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            ue_st.completion_timeout_status.display(BoolView::PlusMinus),
            ue_st.completer_abort_status.display(BoolView::PlusMinus),
            ue_st.unexpected_completion_status.display(BoolView::PlusMinus),
            ue_st.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            ue_st.malformed_tlp_status.display(BoolView::PlusMinus),
            ue_st.ecrc_error_status.display(BoolView::PlusMinus),
            ue_st.unsupported_request_error_status.display(BoolView::PlusMinus),
            ue_st.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tUEMsk:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            ue_msk.data_link_protocol_error_status.display(BoolView::PlusMinus),
            ue_msk.surprise_down_error_status.display(BoolView::PlusMinus),
            ue_msk.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            ue_msk.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            ue_msk.completion_timeout_status.display(BoolView::PlusMinus),
            ue_msk.completer_abort_status.display(BoolView::PlusMinus),
            ue_msk.unexpected_completion_status.display(BoolView::PlusMinus),
            ue_msk.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            ue_msk.malformed_tlp_status.display(BoolView::PlusMinus),
            ue_msk.ecrc_error_status.display(BoolView::PlusMinus),
            ue_msk.unsupported_request_error_status.display(BoolView::PlusMinus),
            ue_msk.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tUESvrt:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            ue_svrt.data_link_protocol_error_status.display(BoolView::PlusMinus),
            ue_svrt.surprise_down_error_status.display(BoolView::PlusMinus),
            ue_svrt.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            ue_svrt.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            ue_svrt.completion_timeout_status.display(BoolView::PlusMinus),
            ue_svrt.completer_abort_status.display(BoolView::PlusMinus),
            ue_svrt.unexpected_completion_status.display(BoolView::PlusMinus),
            ue_svrt.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            ue_svrt.malformed_tlp_status.display(BoolView::PlusMinus),
            ue_svrt.ecrc_error_status.display(BoolView::PlusMinus),
            ue_svrt.unsupported_request_error_status.display(BoolView::PlusMinus),
            ue_svrt.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "\t\tCESta:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}",
            ce_st.receiver_error_status.display(BoolView::PlusMinus),
            ce_st.bad_tlp_status.display(BoolView::PlusMinus),
            ce_st.bad_dllp_status.display(BoolView::PlusMinus),
            ce_st.replay_num_rollover_status.display(BoolView::PlusMinus),
            ce_st.replay_timer_timeout_status.display(BoolView::PlusMinus),
            ce_st.advisory_non_fatal_error_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tCEMsk:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}\n",
            ce_msk.receiver_error_status.display(BoolView::PlusMinus),
            ce_msk.bad_tlp_status.display(BoolView::PlusMinus),
            ce_msk.bad_dllp_status.display(BoolView::PlusMinus),
            ce_msk.replay_num_rollover_status.display(BoolView::PlusMinus),
            ce_msk.replay_timer_timeout_status.display(BoolView::PlusMinus),
            ce_msk.advisory_non_fatal_error_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tAERCap:\tFirst Error Pointer: {:02x}, ECRCGenCap{} ECRCGenEn{} ECRCChkCap{} ECRCChkEn{}\n",
            aer_caps.first_error_pointer,
            aer_caps.ecrc_generation_capable.display(BoolView::PlusMinus),
            aer_caps.ecrc_generation_enable.display(BoolView::PlusMinus),
            aer_caps.ecrc_check_capable.display(BoolView::PlusMinus),
            aer_caps.ecrc_check_enable.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\t\tMultHdrRecCap{} MultHdrRecEn{} TLPPfxPres{} HdrLogCap{}\n",
            aer_caps.multiple_header_recording_capable.display(BoolView::PlusMinus),
            aer_caps.multiple_header_recording_enable.display(BoolView::PlusMinus),
            aer_caps.tlp_prefix_log_present.display(BoolView::PlusMinus),
            aer_caps.completion_timeout_prefix_or_header_log_capable.display(BoolView::PlusMinus),
        )?;
        write!(f, "\t\tHeaderLog: {:08x} {:08x} {:08x} {:08x}\n", hl.0[0], hl.0[1], hl.0[2], hl.0[3])?;
        if let (true, Some(root_cmd), Some(root_st), Some(esi)) = (is_type_root, root_cmd, root_st, esi) {
            write!(f,
                "\t\tRootCmd: CERptEn{} NFERptEn{} FERptEn{}\n",
                root_cmd.correctable_error_reporting_enable.display(BoolView::PlusMinus),
                root_cmd.non_fatal_error_reporting_enable.display(BoolView::PlusMinus),
                root_cmd.fatal_error_reporting_enable.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\tRootSta: CERcvd{} MultCERcvd{} UERcvd{} MultUERcvd{}\n",
                root_st.err_cor_received.display(BoolView::PlusMinus),
                root_st.multiple_err_cor_received.display(BoolView::PlusMinus),
                root_st.err_fatal_or_nonfatal_received.display(BoolView::PlusMinus),
                root_st.multiple_err_fatal_or_nonfatal_received.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\t\t FirstFatal{} NonFatalMsg{} FatalMsg{} IntMsg {}\n",
                root_st.first_uncorrectable_fatal.display(BoolView::PlusMinus),
                root_st.non_fatal_error_messages_received.display(BoolView::PlusMinus),
                root_st.fatal_error_messages_received.display(BoolView::PlusMinus),
                root_st.advanced_error_interrupt_message_number,
            )?;
            write!(f, "\t\tErrorSrc: ERR_COR: {:04x} ", esi.err_cor_source_identification)?;
            write!(f, "ERR_FATAL/NONFATAL: {:04x}\n", esi.err_fatal_or_nonfatal_source_identification)?;
        }
        Ok(())
    }
}
