//! Success mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Success {
    pub safe: bool,
}
impl Success {
    pub fn new(safe: bool) -> Self {
        Self { safe }
    }
}

impl IsMode for Success {
    fn desc(&self) -> &'static str {
        "success"
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            // Done.
            Msg::Status(Status::TlcFinished { runtime: _ }) => {
                out.handle_message(msg, log::Level::Debug);
                Control::Finalize(ModeOutcome::new_success(self.safe)).ok_some()
            }
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcProgressStats { .. }))
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcStats { .. }))
            | Msg::Tlc(TlcMsg::Msg(
                Tlc::TlcSearchDepth { .. } | Tlc::TlcStateGraphOutdegree { .. },
            )) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::keep(self).ok_some()
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
