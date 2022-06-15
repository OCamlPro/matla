//! Handles TLC runs.

prelude!();

use tlc::code;

pub mod utils;

mod control;

mod analysis;
mod error;
mod initial_states;
mod parsing;
mod starting;
mod success;
mod trace;
mod warmup;

pub use self::{
    analysis::Analysis, control::Control, error::Error, initial_states::InitialStates,
    parsing::Parsing, starting::Starting, success::Success, trace::Trace, warmup::WarmUp,
};

/// Variants of mode finalization.
#[derive(Debug, Clone)]
pub enum ModeOutcomeKind {
    /// Nothing failed.
    Success { safe: bool },
    /// Something caused a problem.
    Problem {
        outcome: tlc::FailedOutcome,
        reported: bool,
    },
    /// A counterexample.
    Cex(cex::Cex),
}
impl ModeOutcomeKind {
    /// CEX variant constructor.
    pub fn cex(cex: impl Into<cex::Cex>) -> Self {
        Self::Cex(cex.into())
    }

    /// Triggers an action on an unsafe outcome.
    pub fn map_unsafe<Out>(&self, action: impl FnOnce() -> Out) -> Option<Out> {
        match self {
            Self::Success { safe: false } => Some(action()),
            Self::Success { safe: true } | Self::Cex(_) | Self::Problem { .. } => None,
        }
    }
    /// Map over a problem, if any.
    pub fn map_problem<Out>(
        &self,
        action: impl FnOnce(&tlc::FailedOutcome, bool) -> Out,
    ) -> Option<Out> {
        match self {
            Self::Success { .. } | Self::Cex(_) => None,
            Self::Problem { outcome, reported } => Some(action(outcome, *reported)),
        }
    }

    /// Text description of an outcome kind.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Success { .. } => "success",
            Self::Problem { .. } => "problem",
            Self::Cex(_) => "cex",
        }
    }

    /// Destructs a CEX, error if `self`'s not a [`Self::Cex`].
    pub fn destruct_cex(self) -> Res<cex::Cex> {
        match self {
            Self::Cex(cex) => Ok(cex),
            kind => bail!(
                "expected `ModeOutcomeKind::Cex`, got `{}` variant",
                kind.desc(),
            ),
        }
    }
}

/// Outcome of a mode finalization.
#[derive(Debug, Clone)]
pub struct ModeOutcome {
    pub kind: ModeOutcomeKind,
    pub runtime_trace: Vec<String>,
}
impl ModeOutcome {
    /// Constructor.
    pub fn new(kind: impl Into<ModeOutcomeKind>) -> Self {
        Self {
            kind: kind.into(),
            runtime_trace: vec![],
        }
    }
    /// Error variant constructor.
    pub fn new_problem(outcome: tlc::FailedOutcome, reported: bool) -> Self {
        Self::new(ModeOutcomeKind::Problem { outcome, reported })
    }
    /// CEX variant constructor.
    pub fn new_cex(cex: impl Into<cex::Cex>) -> Self {
        ModeOutcomeKind::cex(cex).into()
    }
    /// Success variant constructor.
    pub fn new_success(safe: bool) -> Self {
        ModeOutcomeKind::Success { safe }.into()
    }
    /// Safe variant constructor.
    pub fn new_safe() -> Self {
        Self::new_success(true)
    }
    /// Unsafe variant constructor.
    pub fn new_unsafe() -> Self {
        Self::new_success(false)
    }
}
implem! {
    for ModeOutcome {
        Deref<Target = Vec<String>> {
            |&self| &self.runtime_trace,
            |&mut self| &mut self.runtime_trace,
        }
        From<ModeOutcomeKind> {
            |kind| Self::new(kind),
        }
    }
}

pub trait IsMode: Sized
where
    TlcMode: From<Self>,
{
    fn desc(&self) -> &'static str;
    fn into_mode(self) -> TlcMode {
        self.into()
    }
    fn handle(self, out: &mut impl tlc::Out, msg: &tlc::msg::Msg) -> Res<Option<Control>> {
        log::debug!("currently in {} mode", self.desc());
        log::debug!("- handling {:?}", msg);
        // utils::report_subs(self.desc(), &msg);
        match &msg.code {
            Some(top_msg) => self.handle_top(out, msg, top_msg),
            None => self.handle_plain(out, msg),
        }
    }
    fn handle_plain(self, out: &mut impl tlc::Out, msg: &tlc::msg::Msg) -> Res<Option<Control>> {
        out.handle_message(msg, log::Level::Info);
        Ok(Some(Control::keep(self)))
    }
    fn handle_top(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::TopMsg,
    ) -> Res<Option<Control>> {
        match tlc_msg {
            code::TopMsg::Msg(tlc_msg) => self.handle_msg(out, msg, tlc_msg),
            code::TopMsg::Err(err) => self.handle_error(out, msg, err, false),
        }
    }
    fn handle_error(
        self,
        _out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        _err: &code::Err,
        reported: bool,
    ) -> Res<Option<Control>> {
        Control::keep_and(self, Error::new(msg.clone(), reported)).ok_some()
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>>
    where
        Self: Sized;

    fn integrate(self, _out: &mut impl tlc::Out, mut outcome: ModeOutcome) -> Res<Control> {
        let desc = outcome.kind.desc();
        outcome.push(format!(
            "unexpected outcome `{}` for mode `{}`",
            desc,
            self.desc(),
        ));
        Control::finalize(outcome).ok()
    }

    fn report_unexpected(&self, msg: &tlc::msg::Msg) {
        utils::report_unexpected(self.desc(), msg)
    }
    fn code_of<'msg>(&self, msg: &'msg tlc::msg::Msg) -> Res<&'msg tlc::code::Msg> {
        utils::code_of(self.desc(), msg)
    }
    fn report_subs(&self, msg: &tlc::msg::Msg) {
        utils::report_subs(self.desc(), msg)
    }
}

macro_rules! build_top {
	(
		$(#[$top_meta:meta])*
		$top_vis:vis enum $top_id:ident {
			$(
				$(#[$variant_meta:meta])*
				$variant_id:ident
			),* $(,)?
		}
	) => {
		$(#[$top_meta])*
		$top_vis enum $top_id {
			$(
				$(#[$variant_meta])*
				$variant_id($variant_id),
			)*
		}
		$(
			impl From<$variant_id> for $top_id {
				fn from(v: $variant_id) -> Self {
					Self::$variant_id(v)
				}
			}
		)*
		impl $top_id {
			/// Mode description.
			pub fn desc(&self) -> &'static str {
				match self {
					$(Self::$variant_id(mode) => mode.desc(),)*
				}
			}
			/// Handles a TLC message.
			pub fn handle(
				self,
				out: &mut impl tlc::Out,
				msg: &tlc::msg::Msg
			) -> Res<Option<Control>> {
				match self {
					$(Self::$variant_id(mode) => mode.handle(out, msg),)*
				}
			}
		    /// Integrates a mode outcome.
		    pub fn integrate(self, out: &mut impl tlc::Out, outcome: ModeOutcome) -> Res<Control> {
		    	match self {
		    		$(Self::$variant_id(mode) => mode.integrate(out, outcome),)*
		    	}
		    }
		}
	};
}

build_top! {
    /// Aggregates all runtime modes.
    #[derive(Debug, Clone)]
    pub enum TlcMode {
        WarmUp,
        Parsing,
        Starting,
        InitialStates,
        Analysis,

        Trace,
        Error,

        Success,
    }
}

implem! {
    for TlcMode {
        From<code::Err> { |err| match err {
            _ => todo!(),
        } }
    }
}

/// A frame of the [`Runtime`] stack.
#[derive(Debug, Clone)]
pub struct Frame {
    /// TLC mode of the frame.
    pub mode: TlcMode,
    /// Instant at which the mode started.
    pub start: time::Instant,
}
implem! {
    impl(Mode: Into<TlcMode>) for Frame {
        From<Mode> { |mode| Self::new(mode) }
    }
}
impl Frame {
    /// Constructor.
    pub fn new(mode: impl Into<TlcMode>) -> Self {
        Self {
            mode: mode.into(),
            start: time::Instant::now(),
        }
    }
    /// Runtime of the frame's mode.
    pub fn runtime(&self) -> time::Duration {
        time::Instant::now() - self.start
    }
}

/// Handles the stack of mode and incoming TLC messages.
pub struct Runtime {
    /// Stack of modes.
    pub stack: SVec<[Frame; 8]>,
    /// Stack memory, used when popping the stack to find someone able to handle a message.
    pub stack_mem: SVec<[Frame; 8]>,
    /// Updated on failures.
    pub outcome: tlc::RunOutcome,
}
impl Runtime {
    /// Constructor.
    pub fn init() -> Self {
        Self {
            stack: smallvec![WarmUp.into()],
            stack_mem: smallvec![],
            outcome: tlc::RunOutcome::Success,
        }
    }

    /// Pushes a frame on the stack.
    fn push(&mut self, frame: Frame) {
        // println!("+ `{}`", frame.mode.desc());
        self.stack.push(frame);
    }
    /// Pops a frame on the stack.
    fn pop(&mut self) -> Option<Frame> {
        let res = self.stack.pop();
        // if let Some(frame) = res.as_ref() {
        //     println!("- `{}`", frame.mode.desc());
        // }
        res
    }

    #[allow(dead_code)]
    fn finalize_stack_mem(&mut self) -> Option<ModeOutcome> {
        todo!("`stack_mem` finalization")
    }
    fn apply_stack_mem(&mut self) {
        self.stack.extend(self.stack_mem.drain(0..).rev())
    }

    /// String description of the modes on the stack in reverse order (bottom-up).
    pub fn stack_desc(&self, other: Option<&str>) -> String {
        self.stack_mem
            .iter()
            .map(|frame| frame.mode.desc())
            .chain(other)
            .chain(self.stack.iter().rev().map(|frame| frame.mode.desc()))
            .fold(String::with_capacity(142), |mut acc, desc| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(desc);
                acc
            })
    }

    /// Handles a message.
    pub fn handle(
        &mut self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
    ) -> Res<Option<tlc::RunOutcome>> {
        if !self.stack_mem.is_empty() {
            bail!("[fatal] invariant falsified: TLC runtime should have an empty `stack_mem`")
        }
        // println!("modes: {}", self.stack_desc(None));
        // println!("- msg: {:?}", msg);
        let res = self.inner_handle(out, msg);
        self.apply_stack_mem();
        res
    }
    fn inner_handle(
        &mut self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
    ) -> Res<Option<tlc::RunOutcome>> {
        // println!();
        // println!("message: {}", msg);
        let Frame { mode, mut start } = self
            .pop()
            .ok_or_else(|| anyhow!("trying to handle TLC message but runtime stack is empty"))?;

        let mut desc = mode.desc();
        let mut control = mode
            .handle(out, msg)
            .with_context(|| anyhow!("in mode `{}`", desc))
            .with_context(|| anyhow!("handling message\n{}", msg))?;

        // This loop keeps `start`, `desc`, and `control` updated. As we go up the stack, we can
        // get new orders to go up ([`Control::Finalize`]), in which case we `continue 'go_up`.
        'go_up: loop {
            // println!("current mode: `{}`", desc);
            // println!("stack: [{}]", self.stack_desc(Some(desc)));
            match control {
                Some(Control::Keep(fst, snd_opt)) => {
                    // println!("keep{}", if snd_opt.is_some() { " and then" } else { "" });
                    self.push(Frame { mode: fst, start });
                    if let Some(snd) = snd_opt {
                        self.push(Frame::new(snd));
                    }
                    break 'go_up Ok(None);
                }
                Some(Control::Replace(mode)) => {
                    // println!("replace");
                    self.push(Frame::new(mode));
                    break 'go_up Ok(None);
                }
                Some(Control::Ignored(mode)) => {
                    // println!("ignored");
                    self.stack_mem.push(Frame { start, mode });
                    if let Some(Frame {
                        mode,
                        start: nu_start,
                    }) = self.pop()
                    {
                        start = nu_start;
                        desc = mode.desc();
                        control = mode
                            .handle(out, msg)
                            .with_context(|| anyhow!("in mode `{}`", desc))?;
                        continue 'go_up;
                    } else {
                        utils::report_unexpected(&self.stack_desc(None), msg);
                        break 'go_up Ok(None);
                    }
                }
                Some(Control::Finalize(outcome)) => {
                    // println!("finalize");
                    // let mode_time = time::Instant::now() - start;
                    // println!(
                    //     "mode `{}` is done in {}.{} seconds",
                    //     desc,
                    //     c.as_secs(),
                    //     mode_time.subsec_millis()
                    // );
                    outcome.kind.map_problem(|outcome, _| {
                        self.outcome
                            .update(&tlc::RunOutcome::Failure(outcome.clone()))
                    });
                    outcome.kind.map_unsafe(|| {
                        self.outcome
                            .update(&tlc::RunOutcome::Failure(tlc::FailedOutcome::Unsafe))
                    });
                    if let Some(Frame {
                        mode,
                        start: nu_start,
                    }) = self.pop()
                    {
                        start = nu_start;
                        desc = mode.desc();
                        control = Some(
                            mode.integrate(out, outcome)
                                .with_context(|| anyhow!("integrating in mode `{}`", desc))?,
                        );
                        continue 'go_up;
                    } else {
                        break 'go_up Ok(Some(self.outcome.clone()));
                    }
                }
                None => todo!("mode `{}`'s handling yielded `None`", desc),
            }
        }
    }
}
