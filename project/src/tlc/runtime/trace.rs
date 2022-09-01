//! Trace (CEX) mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Trace {
    /// The trace of states.
    pub cex: cex::Cex,
}

impl Trace {
    /// Empty CEX constructor.
    pub fn new_empty() -> Self {
        Self::new(cex::Cex::new())
    }
    /// Constructor from a CEX.
    pub fn new(cex: cex::Cex) -> Self {
        Self { cex }
    }
}

impl IsMode for Trace {
    fn desc(&self) -> &'static str {
        "trace"
    }
    fn handle_error(
        self,
        _out: &mut impl tlc::Out,
        _msg: &tlc::msg::Msg,
        _err: &code::Err,
        _reported: bool,
    ) -> Res<Option<Control>> {
        Control::ignored(self).ok_some()
    }
    fn handle_msg(
        mut self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            Msg::Status(Status::TlcCounterExample)
            | Msg::Status(Status::TlcBehaviorUpToThisPoint)
            => {
                out.handle_message(&msg, log::Level::Trace);
                Control::keep(self).ok_some()
            }

            // End of temporal cex.
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcCheckingTemporalPropsEnd))
            // End of normal cex. Note how inappropriate this message is.
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcProgressStats { .. })) => {
                out.handle_message(&msg, log::Level::Trace);
                let outcome = ModeOutcome::new_cex(self.cex);
                Control::finalize(outcome).ok_some()
            }

            // End of temporal cex, stuttering case.
            Msg::Cex(TlcCex::TlcStuttering) => {
                out.handle_message(&msg, log::Level::Trace);
                self.cex = self.cex.shape(cex::Shape::Stuttering);
                Control::keep(self).ok_some()
            }

            // Lasso temporal cex.
            Msg::Cex(TlcCex::TlcBackToState { index }) => {
                out.handle_message(&msg, log::Level::Trace);
                let idx =
                    self.cex.idx_of(*index)?;
                self.cex = self.cex.shape(cex::Shape::Loop(idx));
                Control::keep(self).ok_some()
            }

            // State of a trace.
            Msg::Cex(TlcCex::TlcTraceState { index, state }) => {
                out.handle_message(&msg, log::Level::Trace);
                // Cloning the state here, could be better with a swap or something.
                let _idx = self.cex.push(state.clone());
                if *index == 0 {
                    bail!("unexpected state index `{}`", index);
                }
                let check_idx = self.cex.index_from_usize(index - 1);
                if Some(_idx) != check_idx {
                    bail!(
                        "error handling TLC cex: expected state #{}, got #{}",
                        _idx,
                        index - 1,
                    )
                } else {
                    Control::keep(self).ok_some()
                }
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
