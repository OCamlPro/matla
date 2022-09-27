//! Handles TLC runs.
//!
//! The [`Runtime`] handles the actual run. Doing so is a bit tricky. Basically TLC goes through a
//! few run-modes such as *parsing*, *lexical analysis*, *parsing*, *semantic analysis*, *actual
//! analysis*...
//!
//! # Modes
//!
//! Also, importantly, there is an *error* mode. Errors can be complex to parse and feature several
//! lines of content and sub-messages.
//!
//! To organize things a bit, we reflect (an abstract version of) these modes in the sub-modules. In
//! (rough) temporal order when TLC runs: [`warmup`], [`parsing`], [`starting`], [`initial_states`],
//! [`analysis`], [`success`].
//!
//! That's on no-error runs at least. At any point, we can enter the [`error`] mode to parse an
//! error. On a counterexample, we enter the [`trace`] mode which handles cex parsing.
//!
//! All modes must implement the [`IsMode`] trait. Modes are enumerated by the [`TlcMode`] enum.
//!
//! # Mode stack
//!
//! When parsing an error, we generally lack context to actually construct the user-facing
//! version. So the [`Runtime`] maintains a stack of modes; when [`error`] produces an error, it
//! goes up the stack to be augmented with the relevant info so that it makes sense for users.
//!
//! The [`Runtime`]'s stack is there for another reason: messages can be entangled. We can have
//! [`parsing`] or [`analysis`] messages while parsing an error for instance. So, when the runtime
//! needs to handle a message, it goes through the mode stack until one that can handle the message
//! is found.
//!
//! # Controling the runtime
//!
//! Whenever a mode is queried for handling a message, an error, or anything at all, **ownership of
//! the mode** is transferred to the query. The query then results in a [`control::Control`]
//! instruction for the runtime. This can cause an early-exit, put the mode back in the stack,
//! replace it, put it back but also push a new mode...

prelude!();

use tlc::code;

pub mod utils;

pub mod control;

pub mod analysis;
pub mod error;
pub mod initial_states;
pub mod parsing;
pub mod starting;
pub mod success;
pub mod trace;
pub mod warmup;

pub use self::{
    analysis::Analysis, control::Control, error::Error, initial_states::InitialStates,
    parsing::Parsing, starting::Starting, success::Success, trace::Trace, warmup::WarmUp,
};

/// Trait implemented by runtime modes.
///
/// Many of the functions in this trait have default implementations. Be careful to override the
/// appropriate ones when implementing `IsMode`.
pub trait IsMode: Sized
where
    TlcMode: From<Self>,
{
    /// A **concise** (ideally one-word) description of the mode.
    fn desc(&self) -> &'static str;

    /// Turns itself in a `TlcMode`.
    fn into_mode(self) -> TlcMode {
        self.into()
    }

    // fn finalize(self, current: Option<ModeOutcome>) -> ModeOutcome {
    //     if let Some(out) = current {
    //         out
    //     } else {
    //         ModeOutcomeKind::Unknown.into()
    //     }
    // }

    /// Handles a Tlc-level message.
    fn handle(self, out: &mut impl tlc::Out, msg: &tlc::msg::Msg) -> Res<Option<Control>> {
        log::debug!("currently in {} mode", self.desc());
        log::debug!("- handling {:?}", msg);
        // utils::report_subs(self.desc(), &msg);
        match &msg.code {
            Some(top_msg) => self.handle_top(out, msg, top_msg),
            None => self.handle_plain(out, msg),
        }
    }

    /// Handles a plain-text message.
    fn handle_plain(self, out: &mut impl tlc::Out, msg: &tlc::msg::Msg) -> Res<Option<Control>> {
        out.handle_message(msg, log::Level::Info);
        Ok(Some(Control::keep(self)))
    }

    /// Handles a top-level message.
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

    /// Handles an error.
    fn handle_error(
        self,
        _out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        _err: &code::Err,
        reported: bool,
    ) -> Res<Option<Control>> {
        Control::keep_and(self, Error::new(msg.clone(), reported)).ok_some()
    }

    /// Handles a normal TLC message.
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>>
    where
        Self: Sized;

    /// Integrates an outcome for a sub-mode into itself.
    ///
    /// Can augment the outcome and issue an early-exit [`Control`] instruction, take the outcome
    /// into account and keep going, or whatever it wants.
    fn integrate(self, _out: &mut impl tlc::Out, mut outcome: ModeOutcome) -> Res<Control> {
        let desc = outcome.kind.desc();
        outcome.push(format!(
            "unexpected outcome `{}` for mode `{}`",
            desc,
            self.desc(),
        ));
        Control::finalize(outcome).ok()
    }

    /// Reports an unexpected message.
    fn report_unexpected(&self, msg: &tlc::msg::Msg) {
        utils::report_unexpected(self.desc(), msg)
    }

    /// Extracts the TLC-message of a [`tlc::msg::Msg`], failing if none.
    fn code_of<'msg>(&self, msg: &'msg tlc::msg::Msg) -> Res<&'msg tlc::code::Msg> {
        utils::code_of(self.desc(), msg)
    }

    /// Reports unexpected sub-messages of a message.
    fn report_subs(&self, msg: &tlc::msg::Msg) {
        utils::report_subs(self.desc(), msg)
    }
}

/// This macro automatically builds the [`TlcMode`].
///
/// Its input is pretty much exactly the enum definition, but the *"variants"* are just runtime
/// modes. For instance, writing `Mode` as a variant produces the variant `Mode(Mode)`.
///
/// - implements `From<Mode>` for all modes;
/// - implements `From<code::Err>`;
/// - lifts [`IsMode::desc`], [`IsMode::handle`] and [`IsMode::integrate`].
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

        implem! {
            for $top_id {
                From<code::Err> { |err| match err {
                    _ => todo!(),
                } }
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

/// A frame of the [`Runtime`] stack, which is just a [`TlcMode`] with some stats.
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
    /// Runtime (until now) of the frame's mode.
    pub fn runtime(&self) -> time::Duration {
        time::Instant::now() - self.start
    }
}

/// Handles the stack of mode and incoming TLC messages.
pub struct Runtime {
    /// Stack of modes.
    pub stack: SVec<[Frame; 8]>,
    /// Stack memory, used when popping the stack to find someone able to handle a message.
    ///
    /// This should **always** be empty outside of [`Runtime::handle`].
    pub stack_mem: SVec<[Frame; 8]>,
    /// Updated on errors / cex-s.
    pub outcome: RunOutcome,
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

    // #[allow(dead_code)]
    // fn finalize_stack_mem(&mut self) -> Option<ModeOutcome> {
    //     todo!("`stack_mem` finalization")
    // }

    /// Puts back `self.stack_mem` on `self.stack` in the right order.
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

    /// Fold over all the errors in the mode stack.
    pub fn tlc_error_fold<Acc>(
        &self,
        mut fold: impl FnMut(Acc, tlc::err::TlcError, bool) -> Res<Acc>,
        init: Acc,
    ) -> Res<Acc> {
        self.error_fold(
            |acc, err| {
                let (err, reported) = err.clone().into_error()?;
                fold(acc, err, reported)
            },
            init,
        )
    }

    /// Fold over all the errors in the mode stack.
    pub fn error_fold<Acc>(
        &self,
        mut fold: impl FnMut(Acc, &error::Error) -> Res<Acc>,
        mut init: Acc,
    ) -> Res<Acc> {
        for frame in self.stack.iter().rev() {
            match &frame.mode {
                TlcMode::Error(err) => init = fold(init, err)?,
                _ => (),
            }
        }
        Ok(init)
    }

    /// Handles a message.
    ///
    /// Goes up the mode stack to find someone that can handle the message using `out`.
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
