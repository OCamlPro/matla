//! Warm up mode, initial TLC runtime mode.

use super::*;

#[derive(Debug, Clone)]
pub struct WarmUp;

impl IsMode for WarmUp {
    fn desc(&self) -> &'static str {
        "warmup"
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            // Going to parsing.
            Msg::Status(Status::TlcSanyStart) => {
                out.handle_message(msg, log::Level::Debug);
                Control::replace(Parsing::new()).ok_some()
            }
            // Version info.
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcVersion)) | Msg::Tlc(TlcMsg::Msg(Tlc::TlcModeMc)) => {
                out.handle_message(msg, log::Level::Trace);
                Control::keep(self).ok_some()
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
