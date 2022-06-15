//! Starting mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Starting;

impl IsMode for Starting {
    fn desc(&self) -> &'static str {
        "starting"
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
            Msg::Status(Status::TlcComputingInit) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::replace(InitialStates).ok_some()
            }
            Msg::Tlc(TlcMsg::Live(TlcLive::TlcLiveImplied)) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::keep(self).ok_some()
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
