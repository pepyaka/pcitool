use core::fmt;

use pcics::extended_capabilities::AdvancedErrorReporting;
use crate::{
    view::{
        DisplayMultiViewBasic,
        MultiView,
        BoolView
    },
};



pub struct AerView {
    pub verbose: usize,
    pub is_type_root: bool,
}


impl<'a> DisplayMultiViewBasic<AerView> for AdvancedErrorReporting {}
impl<'a> fmt::Display for MultiView<&'a AdvancedErrorReporting, AerView> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AerView { verbose, is_type_root } = self.view;
        writeln!(f, "Advanced Error Reporting")?;
        let AdvancedErrorReporting {
            uncorrectable_error_status: uesta,
            uncorrectable_error_mask: uemsk,
            uncorrectable_error_severity: uesvrt,
            correctable_error_status: cesta,
            correctable_error_mask: cemsk,
            advanced_error_capabilities_and_control: aercap,
            header_log: hl,
            root_error_command: rootcmd,
            root_error_status: rootsta,
            error_source_identification: esi,
            ..
        } = self.data;
        if verbose < 2 {
            return Ok(());
        }
        write!(f,
            "\t\tUESta:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            uesta.data_link_protocol_error_status.display(BoolView::PlusMinus),
            uesta.surprise_down_error_status.display(BoolView::PlusMinus),
            uesta.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            uesta.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            uesta.completion_timeout_status.display(BoolView::PlusMinus),
            uesta.completer_abort_status.display(BoolView::PlusMinus),
            uesta.unexpected_completion_status.display(BoolView::PlusMinus),
            uesta.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            uesta.malformed_tlp_status.display(BoolView::PlusMinus),
            uesta.ecrc_error_status.display(BoolView::PlusMinus),
            uesta.unsupported_request_error_status.display(BoolView::PlusMinus),
            uesta.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tUEMsk:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            uemsk.data_link_protocol_error_status.display(BoolView::PlusMinus),
            uemsk.surprise_down_error_status.display(BoolView::PlusMinus),
            uemsk.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            uemsk.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            uemsk.completion_timeout_status.display(BoolView::PlusMinus),
            uemsk.completer_abort_status.display(BoolView::PlusMinus),
            uemsk.unexpected_completion_status.display(BoolView::PlusMinus),
            uemsk.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            uemsk.malformed_tlp_status.display(BoolView::PlusMinus),
            uemsk.ecrc_error_status.display(BoolView::PlusMinus),
            uemsk.unsupported_request_error_status.display(BoolView::PlusMinus),
            uemsk.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tUESvrt:\tDLP{} SDES{} TLP{} FCP{} CmpltTO{} CmpltAbrt{} UnxCmplt{} RxOF{} ",
            uesvrt.data_link_protocol_error_status.display(BoolView::PlusMinus),
            uesvrt.surprise_down_error_status.display(BoolView::PlusMinus),
            uesvrt.poisoned_tlp_received_status.display(BoolView::PlusMinus),
            uesvrt.flow_control_protocol_error_status.display(BoolView::PlusMinus),
            uesvrt.completion_timeout_status.display(BoolView::PlusMinus),
            uesvrt.completer_abort_status.display(BoolView::PlusMinus),
            uesvrt.unexpected_completion_status.display(BoolView::PlusMinus),
            uesvrt.receiver_overflow_status.display(BoolView::PlusMinus),
        )?;
        writeln!(f,
            "MalfTLP{} ECRC{} UnsupReq{} ACSViol{}",
            uesvrt.malformed_tlp_status.display(BoolView::PlusMinus),
            uesvrt.ecrc_error_status.display(BoolView::PlusMinus),
            uesvrt.unsupported_request_error_status.display(BoolView::PlusMinus),
            uesvrt.acs_violation_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tCESta:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}\n",
            cesta.receiver_error_status.display(BoolView::PlusMinus),
            cesta.bad_tlp_status.display(BoolView::PlusMinus),
            cesta.bad_dllp_status.display(BoolView::PlusMinus),
            cesta.replay_num_rollover_status.display(BoolView::PlusMinus),
            cesta.replay_timer_timeout_status.display(BoolView::PlusMinus),
            cesta.advisory_non_fatal_error_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tCEMsk:\tRxErr{} BadTLP{} BadDLLP{} Rollover{} Timeout{} AdvNonFatalErr{}\n",
            cemsk.receiver_error_status.display(BoolView::PlusMinus),
            cemsk.bad_tlp_status.display(BoolView::PlusMinus),
            cemsk.bad_dllp_status.display(BoolView::PlusMinus),
            cemsk.replay_num_rollover_status.display(BoolView::PlusMinus),
            cemsk.replay_timer_timeout_status.display(BoolView::PlusMinus),
            cemsk.advisory_non_fatal_error_status.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\tAERCap:\tFirst Error Pointer: {:02x}, ECRCGenCap{} ECRCGenEn{} ECRCChkCap{} ECRCChkEn{}\n",
            aercap.first_error_pointer,
            aercap.ecrc_generation_capable.display(BoolView::PlusMinus),
            aercap.ecrc_generation_enable.display(BoolView::PlusMinus),
            aercap.ecrc_check_capable.display(BoolView::PlusMinus),
            aercap.ecrc_check_enable.display(BoolView::PlusMinus),
        )?;
        write!(f,
            "\t\t\tMultHdrRecCap{} MultHdrRecEn{} TLPPfxPres{} HdrLogCap{}\n",
            aercap.multiple_header_recording_capable.display(BoolView::PlusMinus),
            aercap.multiple_header_recording_enable.display(BoolView::PlusMinus),
            aercap.tlp_prefix_log_present.display(BoolView::PlusMinus),
            aercap.completion_timeout_prefix_or_header_log_capable.display(BoolView::PlusMinus),
        )?;
        write!(f, "\t\tHeaderLog: {:08x} {:08x} {:08x} {:08x}\n", hl.0[0], hl.0[1], hl.0[2], hl.0[3])?;
        if is_type_root {
            write!(f,
                "\t\tRootCmd: CERptEn{} NFERptEn{} FERptEn{}\n",
                rootcmd.correctable_error_reporting_enable.display(BoolView::PlusMinus),
                rootcmd.non_fatal_error_reporting_enable.display(BoolView::PlusMinus),
                rootcmd.fatal_error_reporting_enable.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\tRootSta: CERcvd{} MultCERcvd{} UERcvd{} MultUERcvd{}\n",
                rootsta.err_cor_received.display(BoolView::PlusMinus),
                rootsta.multiple_err_cor_received.display(BoolView::PlusMinus),
                rootsta.err_fatal_or_nonfatal_received.display(BoolView::PlusMinus),
                rootsta.multiple_err_fatal_or_nonfatal_received.display(BoolView::PlusMinus),
            )?;
            write!(f,
                "\t\t\t FirstFatal{} NonFatalMsg{} FatalMsg{} IntMsg {}\n",
                rootsta.first_uncorrectable_fatal.display(BoolView::PlusMinus),
                rootsta.non_fatal_error_messages_received.display(BoolView::PlusMinus),
                rootsta.fatal_error_messages_received.display(BoolView::PlusMinus),
                rootsta.advanced_error_interrupt_message_number,
            )?;
            write!(f, "\t\tErrorSrc: ERR_COR: {:04x} ", esi.err_cor_source_identification)?;
            write!(f, "ERR_FATAL/NONFATAL: {:04x}\n", esi.err_fatal_or_nonfatal_source_identification)?;
        }
        Ok(())
    }
}
