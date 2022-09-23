//! Analysis mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Analysis {
    pub safe: bool,
}
impl Analysis {
    pub fn new() -> Self {
        Self { safe: true }
    }
}

impl IsMode for Analysis {
    fn desc(&self) -> &'static str {
        "analysis"
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
            Msg::Status(Status::TlcSuccess) => {
                out.handle_message(&msg, log::Level::Trace);
                Control::replace(Success::new(self.safe)).ok_some()
            }

            // Regular stats.
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcProgressStats { .. }))
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcStats { .. }))
            // Starting to check temporal properties.
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcCheckingTemporalProps { .. }))
            // Starting to check temporal properties.
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcCheckingTemporalPropsEnd { .. })) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::keep(self).ok_some()
            }

            // Cex on invariant.
            Msg::TlcUnsafe(TlcUnsafe::TlcInvariantViolatedBehavior { invariant }) => {
                out.handle_message(&msg, log::Level::Debug);
                let cex = cex::Cex::new().set_falsified(invariant)?;
                Control::keep_and(self, Trace::new(cex)).ok_some()
            }
            // Cex on temporal property.
            Msg::TlcUnsafe(TlcUnsafe::TlcTemporalPropertyViolated) => {
                out.handle_message(&msg, log::Level::Debug);
                let cex = cex::Cex::new();
                Control::keep_and(self, Trace::new(cex)).ok_some()
            }

            _ => Control::ignored(self).ok_some(),
        }
    }
    fn integrate(mut self, out: &mut impl tlc::Out, outcome: ModeOutcome) -> Res<Control> {
        match outcome.kind {
            ModeOutcomeKind::Unknown => Control::keep(self).ok(),
            ModeOutcomeKind::Success { safe } => {
                if !safe {
                    self.safe = false
                }
                Control::keep(self).ok()
            }
            ModeOutcomeKind::Cex(cex) => {
                out.handle_cex(cex);
                Control::replace(Success::new(false)).ok()
            }
            ModeOutcomeKind::Problem { reported, .. } => {
                if reported {
                    Control::keep(self).ok()
                } else {
                    bail!("problems should be reported at this point")
                }
            }
        }
    }
}
