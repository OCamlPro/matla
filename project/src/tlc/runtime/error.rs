//! Error mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Error {
    /// Error message.
    pub msgs: SVec<[tlc::msg::Msg; 8]>,
    /// CEX, if any.
    pub trace: Option<cex::Cex>,
    /// If true, the error has already been reported.
    pub reported: bool,
}
impl Error {
    /// Empty constructor.
    pub fn empty(reported: bool) -> Self {
        Self {
            msgs: smallvec![],
            trace: None,
            reported,
        }
    }
    /// Constructor.
    pub fn new(msg: tlc::msg::Msg, reported: bool) -> Self {
        Self {
            reported,
            msgs: smallvec![msg.into()],
            trace: None,
        }
    }

    /// Turns itself into a real TLC error and a flag that's true iff the error has been reported.
    pub fn into_error(self) -> Res<(tlc::TlcError, bool)> {
        let mut msgs = self.msgs.into_iter();
        let first_msg = msgs
            .next()
            .ok_or_else(|| anyhow!("illegal error with no message"))?;
        let (err, subs) = first_msg.into_err()?;

        let mut tlc_error = err.into_tlc_error(subs)?;

        for msg in msgs {
            tlc_error.integrate(msg)?
        }

        if let Some(behavior) = self.trace {
            tlc_error.set_behavior(behavior)?;
        }

        Ok((tlc_error, self.reported))
    }
}

impl IsMode for Error {
    fn desc(&self) -> &'static str {
        "error"
    }

    fn handle_error(
        mut self,
        _out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        _err: &code::Err,
        _reported: bool,
    ) -> Res<Option<Control>> {
        self.msgs.push(msg.clone());
        Control::keep(self).ok_some()
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            Msg::Tlc(TlcMsg::Msg(Tlc::TlcProgressStats { .. }))
            | Msg::Tlc(TlcMsg::Msg(Tlc::TlcStats { .. })) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::keep_and(self, Trace::new_empty()).ok_some()
            }
            Msg::Status(Status::TlcBehaviorUpToThisPoint) => {
                out.handle_message(&msg, log::Level::Trace);
                Control::keep_and(self, Trace::new_empty()).ok_some()
            }
            Msg::Status(Status::TlcFinished { .. }) => {
                let (error, reported) = self.into_error()?;
                let outcome = error
                    .to_outcome()
                    .unwrap_or_else(|| tlc::FailedOutcome::Plain(format!("fatal error")));
                if !reported {
                    out.handle_error(error)?;
                }
                Control::finalize(ModeOutcome::new_problem(outcome, true)).ok_some()
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
    fn integrate(mut self, _out: &mut impl tlc::Out, outcome: ModeOutcome) -> Res<Control> {
        match outcome.kind {
            ModeOutcomeKind::Unknown => Control::keep(self).ok(),
            ModeOutcomeKind::Success { .. } => Control::keep(self).ok(),
            ModeOutcomeKind::Cex(cex) => {
                if self.trace.is_some() {
                    bail!("trying to set trace for error `{}` twice", self.msgs[0]);
                }
                self.trace = Some(cex);
                Control::keep(self).ok()
            }
            kind => bail!(
                "expected `success` or `cex` outcome variant, got `{}`",
                kind.desc()
            ),
        }
    }
}
