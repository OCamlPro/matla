//! Initial states mode.

use super::*;

#[derive(Debug, Clone)]
pub struct InitialStates;

impl IsMode for InitialStates {
    fn desc(&self) -> &'static str {
        "initial_states"
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            // Parsing ends.
            Msg::Status(
                Status::TlcInitGenerated1 { .. }
                | Status::TlcInitGenerated2
                | Status::TlcInitGenerated3
                | Status::TlcInitGenerated4,
            ) => {
                out.handle_message(&msg, log::Level::Trace);
                Control::replace(Analysis::new()).ok_some()
            }
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcComputingInitProgress))
            | Msg::Tlc(TlcMsg::Live(TlcLive::TlcLiveImplied)) => {
                out.handle_message(&msg, log::Level::Trace);
                Control::keep(self).ok_some()
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
