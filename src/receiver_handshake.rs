use std::fmt;

use serde::Serialize;

use crate::protocol::srt_handshake::HsrspInfo;
use crate::protocol::{SRT_OPT_NAKREPORT, SRT_OPT_REXMITFLG};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ReceiverHandshake {
    #[serde(rename = "nak_report", skip_serializing_if = "Option::is_none")]
    pub receiver_nak_report: Option<bool>,
    #[serde(rename = "srt_version", skip_serializing_if = "Option::is_none")]
    pub receiver_srt_version: Option<String>,
    #[serde(rename = "rexmit_flag", skip_serializing_if = "Option::is_none")]
    pub receiver_rexmit_flag: Option<bool>,
}

impl ReceiverHandshake {
    /// Policy consumers must assume periodic NAKs until an HSRSP says otherwise.
    pub const fn nak_report_enabled(&self) -> bool {
        match self.receiver_nak_report {
            Some(enabled) => enabled,
            None => true,
        }
    }
}

impl From<HsrspInfo> for ReceiverHandshake {
    fn from(info: HsrspInfo) -> Self {
        Self {
            receiver_nak_report: Some(info.flags & SRT_OPT_NAKREPORT != 0),
            receiver_srt_version: Some(format!(
                "{}.{}.{}",
                (info.srt_version >> 16) & 0xff,
                (info.srt_version >> 8) & 0xff,
                info.srt_version & 0xff,
            )),
            receiver_rexmit_flag: Some(info.flags & SRT_OPT_REXMITFLG != 0),
        }
    }
}

impl fmt::Display for ReceiverHandshake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nak = match self.receiver_nak_report {
            Some(true) => "on",
            Some(false) => "off",
            None => "unknown",
        };
        write!(
            f,
            "receiver: nak_report={nak} srt={}",
            self.receiver_srt_version.as_deref().unwrap_or("unknown")
        )
    }
}
