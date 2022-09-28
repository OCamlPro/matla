//! Gathers the different kinds of outcome.
//!
//! # Top-level outcome kinds
//!
//! The top-most outcome kinds are
//! - [`Outcome`], stores
//!   - run stats: runtime, start date;
//!   - errors: a list of [`TlcError`]s;
//!   - a [`ProcessOutcome`];
//!   - an optional [`RunOutcome`].
//! - [`ConciseOutcome`], a one-or-two-word description of what happened.
//!
//! [`Outcome`] is used to report what happened in detailed, while [`ConciseOutcome`] is much more
//! abstract. In particular, matla tests specify an expected [`ConciseOutcome`] which matla
//! compares with the actual [`ConciseOutcome`].
//!
//! # Relations between outcome kinds
//!
//! - [`ModeOutcomeKind`]: can be a [`FailedOutcome`];
//! - [`ModeOutcome`]: stores a [`ModeOutcomeKind`];
//! - [`RunOutcome`]: can be a [`FailedOutcome`];
//! - [`Outcome`]: stores a [`ProcessOutcome`], an optional [`RunOutcome`], and zero or more
//!       [`TlcError`]s.
//!
//! [`TlcError`]: tlc::err::TlcError (Type of TLC-level errors)

prelude!();

/// A failed outcome of a TLC run.
///
/// Used by [`ModeOutcomeKind`] and [`RunOutcome`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FailedOutcome {
    /// A parse error with a description.
    ParseError,
    /// An assertion failure with a description.
    AssertFailed,
    /// Deadlock.
    Deadlock,
    /// Unsafe.
    Unsafe,
    /// Plain error.
    Plain(String),
}
impl FailedOutcome {
    /// True if [`FailedOutcome::Deadlock`].
    pub fn is_deadlock(&self) -> bool {
        *self == Self::Deadlock
    }
}
implem! {
    for FailedOutcome {
        Display {
            |&self, fmt| {
                match self {
                    Self::ParseError => "parse error".fmt(fmt),
                    Self::AssertFailed => "assertion failure".fmt(fmt),
                    Self::Deadlock => "deadlock".fmt(fmt),
                    Self::Unsafe => "unsafe".fmt(fmt),
                    Self::Plain(s) => write!(fmt, "<{}>", s),
                }
            }
        }
    }
}

/// Variants of mode finalization.
#[derive(Debug, Clone)]
pub enum ModeOutcomeKind {
    Unknown,
    /// Nothing failed.
    Success {
        safe: bool,
    },
    /// Something caused a problem.
    Problem {
        outcome: FailedOutcome,
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
            Self::Success { safe: true } | Self::Cex(_) | Self::Problem { .. } | Self::Unknown => {
                None
            }
        }
    }
    /// Map over a problem, if any.
    pub fn map_problem<Out>(
        &self,
        action: impl FnOnce(&FailedOutcome, bool) -> Out,
    ) -> Option<Out> {
        match self {
            Self::Success { .. } | Self::Cex(_) | Self::Unknown => None,
            Self::Problem { outcome, reported } => Some(action(outcome, *reported)),
        }
    }

    /// Text description of an outcome kind.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Success { .. } => "success",
            Self::Problem { .. } => "problem",
            Self::Unknown => "unknown",
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
///
/// Stores a `runtime_trace: Vec<String>` which is not used at the moment.
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
    pub fn new_problem(outcome: FailedOutcome, reported: bool) -> Self {
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

/// Outcome of a TLC run.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunOutcome {
    /// Success.
    Success,
    /// Failure.
    Failure(FailedOutcome),
}
impl RunOutcome {
    /// True if success.
    pub fn is_success(&self) -> bool {
        match self {
            Self::Success => true,
            Self::Failure(_) => false,
        }
    }
    /// True if deadlock failure.
    pub fn is_deadlock(&self) -> bool {
        *self == Self::Failure(FailedOutcome::Deadlock)
    }

    /// Turns itself into a failure.
    pub fn into_failure(self) -> Res<FailedOutcome> {
        match self {
            Self::Success => bail!("expected run to fail but got a successful outcome"),
            Self::Failure(outcome) => Ok(outcome),
        }
    }

    /// Map over failures, if any.
    pub fn map_failure<Out>(&self, action: impl FnOnce(&FailedOutcome) -> Out) -> Option<Out> {
        match self {
            Self::Success => None,
            Self::Failure(f) => Some(action(f)),
        }
    }

    /// Updates itself with a new outcome.
    ///
    /// If `self` is [`Self::Success`], replaces itself by `that`. Otherwise, does nothing.
    pub fn update(&mut self, that: &Self) {
        match self {
            Self::Success => {
                *self = that.clone();
            }
            Self::Failure(_) => (),
        }
    }
}
implem! {
    for RunOutcome {
        Display {
            |&self, fmt| match self {
                Self::Success => "success".fmt(fmt),
                Self::Failure(failed) => failed.fmt(fmt),
            }
        }
        From<FailedOutcome> {
            |failure| Self::Failure(failure)
        }
    }
}

/// Outcome of a TLC process: a code and an exit status.
#[derive(Debug, Clone)]
pub struct ProcessOutcome {
    /// Exit code.
    pub code: i32,
    /// TLC exit status.
    pub status: Option<tlc::code::Exit>,
}
impl ProcessOutcome {
    /// Constructor.
    pub fn new(code: i32) -> Res<Self> {
        let status = tlc::code::Exit::from_code(code, &tlc::msg::Elms::EMPTY)?;
        Ok(Self { code, status })
    }
}
implem! {
    for ProcessOutcome {
        Display {
            |&self, fmt| {
                if let Some(status) = self.status.as_ref() {
                    write!(fmt, "{} ({})", status, self.code)
                } else {
                    write!(fmt, "unknown TLC exit code {}", self.code)
                }
            }
        }
    }
}

/// Outcome of a run, including statistics and errors.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// Process outcome.
    pub process: ProcessOutcome,
    /// Run outcome.
    pub run: Option<RunOutcome>,
    /// Runtime.
    pub runtime: chrono::Duration,
    /// Outcome.
    pub errors: Vec<tlc::err::TlcError>,
    /// Start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
}
impl Outcome {
    /// Constructor.
    pub fn new(
        process: ProcessOutcome,
        run: Option<RunOutcome>,
        runtime: chrono::Duration,
        start_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            process,
            run,
            runtime,
            start_time,
            errors: vec![],
        }
    }

    /// Produces a concise outcome for the final report.
    pub fn to_concise(&self) -> ConciseOutcome {
        use tlc::code::Exit;
        match (&self.run, &self.process.status) {
            (Some(RunOutcome::Success), _) | (None, Some(Exit::Success)) => ConciseOutcome::Success,

            (Some(RunOutcome::Failure(FailedOutcome::Unsafe)), _)
            | (Some(RunOutcome::Failure(FailedOutcome::Deadlock)), _)
            | (None, Some(Exit::Violation(_))) => ConciseOutcome::Unsafe,

            (Some(RunOutcome::Failure(FailedOutcome::ParseError)), _)
            | (None, Some(Exit::Failure(_))) => ConciseOutcome::IllDefined,

            (Some(RunOutcome::Failure(FailedOutcome::AssertFailed)), _) => {
                ConciseOutcome::AssertFailed
            }
            (Some(RunOutcome::Failure(FailedOutcome::Plain(err))), _) => {
                ConciseOutcome::Error(Some(err))
            }
            (None, Some(Exit::Error(_) | Exit::PlainError)) => ConciseOutcome::Error(None),

            (None, None) => ConciseOutcome::Unknown,
        }
    }
}
implem! {
    for Outcome {
        Deref<Target = Vec<tlc::err::TlcError>> {
            |&self| &self.errors,
            |&mut self| &mut self.errors,
        }
    }
}

/// Concise version of a run outcome.
///
/// Corresponds to the *final* result reported to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConciseOutcome<'msg> {
    Success,
    Unsafe,
    IllDefined,
    Error(Option<&'msg str>),
    AssertFailed,
    Unknown,
}
implem! {
    impl('msg) for ConciseOutcome<'msg> {
        Display { |&self, fmt| match self {
            Self::Success => "success".fmt(fmt),
            Self::Unsafe => "unsafe".fmt(fmt),
            Self::IllDefined => "ill-defined".fmt(fmt),
            Self::Error(None) => "error".fmt(fmt),
            Self::Error(Some(msg)) => write!(fmt, "error[{}]", msg),
            Self::AssertFailed => "assert failed".fmt(fmt),
            Self::Unknown => "<unknown>".fmt(fmt),
        } }
    }
}
impl<'msg> ConciseOutcome<'msg> {
    /// Description.
    pub fn desc(&self) -> String {
        match self {
            Self::Error(Some(msg)) => format!("error: {}", msg),
            _ => conf::exit_code::desc(self.to_exit_code())
                .expect("[fatal] exit codes and outcomes are desync-ed")
                .into(),
        }
    }

    /// Fails if `self` and `reference` are different, presenting the latter as expected.
    pub fn expecting(self, reference: Self) -> Res<()> {
        if self == reference {
            Ok(())
        } else {
            bail!(
                "expected `{}` outcome, got `{}`",
                reference.desc(),
                self.desc()
            )
        }
    }

    /// True on [`Self::Success`].
    pub fn is_success(self) -> bool {
        self == Self::Success
    }
    /// True on [`Self::Unsafe`].
    pub fn is_unsafe(self) -> bool {
        self == Self::Unsafe
    }
    /// True on [`Self::IllDefined`].
    pub fn is_ill_defined(self) -> bool {
        self == Self::IllDefined
    }
    /// True on [`Self::Error`].
    pub fn is_error(self) -> bool {
        if let Self::Error(_) = self {
            true
        } else {
            false
        }
    }
    /// True on [`Self::AssertFailed`].
    pub fn is_assert_failed(self) -> bool {
        self == Self::AssertFailed
    }
    /// True on [`Self::Unknown`].
    pub fn is_unknown(self) -> bool {
        self == Self::Unknown
    }

    /// Matla exit code associated with this outcome.
    pub fn to_exit_code(self) -> i32 {
        use conf::exit_code::*;
        match self {
            Self::Success => SAFE,
            Self::Unsafe => UNSAFE,
            Self::IllDefined => ILL_DEFINED,
            Self::Error(_) => ERROR,
            Self::AssertFailed => ASSERT_FAILED,
            Self::Unknown => UNKNOWN,
        }
    }
    /// Constructor from an exit code, mostly used for internal tests.
    pub fn from_exit_code(code: i32) -> Res<Self> {
        use conf::exit_code::*;
        let slf = if code == SAFE {
            Self::Success
        } else if code == UNSAFE {
            Self::Unsafe
        } else if code == ILL_DEFINED {
            Self::IllDefined
        } else if code == ERROR {
            Self::Error(None)
        } else if code == ASSERT_FAILED {
            Self::AssertFailed
        } else if code == UNKNOWN {
            Self::Unknown
        } else {
            bail!(
                "matla exit code `{}` does not exist and has no semantics",
                code
            )
        };
        Ok(slf)
    }
}
