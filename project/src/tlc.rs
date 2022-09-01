//! Handles TLC runs.

prelude!();

pub mod code;
pub mod err;
pub mod msg;
pub mod parse;
pub mod runtime;
pub mod warn;

// pub mod initial_states;

pub use err::TlcError;

/// A failed outcome of a TLC run.
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
    /// True if [`Self::Deadlock`].
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

/// Outcome of a TLC run.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunOutcome {
    /// Success.
    Success,
    /// Failure.
    Failure(FailedOutcome),
}
impl RunOutcome {
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
    }
}

/// Outcome of a TLC process.
#[derive(Debug, Clone)]
pub struct ProcessOutcome {
    /// Exit code.
    pub code: i32,
    /// TLC exit status.
    pub status: Option<code::Exit>,
}
impl ProcessOutcome {
    /// Constructor.
    pub fn new(code: i32) -> Res<Self> {
        let status = code::Exit::from_code(code, &msg::Elms::EMPTY)?;
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

#[derive(Debug, Clone)]
pub struct Outcome {
    /// Process outcome.
    pub process: ProcessOutcome,
    /// Run outcome.
    pub run: Option<RunOutcome>,
    /// Runtime.
    pub runtime: chrono::Duration,
    /// Outcome.
    pub errors: Vec<err::TlcError>,
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
        use code::Exit;
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
        Deref<Target = Vec<err::TlcError>> {
            |&self| &self.errors,
            |&mut self| &mut self.errors,
        }
    }
}

/// Concise version of a run outcome.
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

/// Output handler trait.
///
/// Used by [`TlcRun`] to handle TLC's output.
///
/// This is mostly used to retrieve the whole TLC output when testing. In run mode, only non-silent
/// messages are printed.
pub trait Out {
    /// Handles a message.
    fn handle_message(&mut self, msg: &msg::Msg, log_level: log::Level);

    /// Handles a mode outcome.
    fn handle_outcome(&mut self, outcome: tlc::RunOutcome);

    /// Handles an error.
    fn handle_error(&mut self, error: impl Into<tlc::err::TlcError>) -> Res<()>;

    /// Handles a cex.
    fn handle_cex(&mut self, cex: cex::Cex);
}
impl<'a, T: Out> Out for &'a mut T {
    fn handle_message(&mut self, msg: &msg::Msg, log_level: log::Level) {
        (*self).handle_message(msg, log_level)
    }
    fn handle_outcome(&mut self, outcome: tlc::RunOutcome) {
        (*self).handle_outcome(outcome)
    }
    fn handle_error(&mut self, error: impl Into<tlc::err::TlcError>) -> Res<()> {
        (*self).handle_error(error)
    }
    fn handle_cex(&mut self, cex: cex::Cex) {
        (*self).handle_cex(cex)
    }
}
impl Out for () {
    fn handle_message(&mut self, _msg: &msg::Msg, _log_level: log::Level) {}
    fn handle_outcome(&mut self, _outcome: tlc::RunOutcome) {}
    fn handle_error(&mut self, _outcome: impl Into<tlc::err::TlcError>) -> Res<()> {
        Ok(())
    }
    fn handle_cex(&mut self, _cex: cex::Cex) {}
}

/// Handles the whole TLC run.
pub struct TlcRun<O> {
    tlc: msg::TlcHandler,
    tlc_lines: Option<Vec<String>>,
    out_handler: O,
    runtime: runtime::Runtime,
}
impl<O: Out> TlcRun<O> {
    /// Constructor.
    pub fn new(cmd: io::Command, out_handler: O) -> Self {
        log::debug!("running TLC with {:?}", cmd);
        let tlc = msg::TlcHandler::new(cmd);
        Self {
            tlc,
            tlc_lines: None,
            out_handler,
            runtime: runtime::Runtime::init(),
        }
    }

    /// Activates line collection from TLC's output.
    pub fn collect_tlc_lines(mut self) -> Self {
        self.tlc_lines = Some(Vec::with_capacity(113));
        self
    }

    /// Retrieves TLC's output.
    pub fn tlc_lines(&self) -> Option<&[String]> {
        self.tlc_lines.as_ref().map(Vec::as_slice)
    }
    /// Drains TLC's output.
    pub fn drain_tlc_lines(&mut self) -> Option<Vec<String>> {
        let mut res = mem::replace(&mut self.tlc_lines, None);
        res.as_mut().map(Vec::shrink_to_fit);
        res
    }

    /// Handles TLC's output.
    pub fn run(mut self) -> Res<Outcome> {
        let mut err: Option<base::Error> = None;
        let mut outcome = None;
        let start_time = chrono::Utc::now();

        'doit: loop {
            // We can't return right away when there's an error. We must `join` with `self.tlc`,
            // otherwise we'll end up with dangling processes or something. So, on error, update
            // `err`, break, and let the post-loop code handle everything.
            macro_rules! try_break {
                ($e:expr) => {
                    match $e {
                        Ok(res) => res,
                        Err(e) => {
                            err = Some(e.into());
                            break 'doit;
                        }
                    }
                };
            }

            let msg = match try_break!(self.tlc.next()) {
                Some(msg) => msg,
                None => {
                    break 'doit;
                }
            };
            let maybe_done = try_break!(self.runtime.handle(&mut self.out_handler, &msg));
            if let Some(nu_outcome) = maybe_done {
                self.out_handler.handle_outcome(nu_outcome.clone());
                outcome = Some(nu_outcome);
                break 'doit;
            }
        }
        let runtime = chrono::Utc::now() - start_time;
        let res = if outcome.is_some() {
            self.tlc.destroy()
        } else {
            self.tlc.join()
        };
        if let Some(err) = err {
            Err(err)
        } else {
            res.map(|raw| Outcome::new(raw, outcome, runtime, start_time))
        }
    }
}

/// A temporal trace that can be produced by TLC.
pub struct Trace {
    pub states: Map<usize, TraceState>,
}
pub struct TraceState {
    pub idx: usize,
    pub header: Option<(Span, String)>,
    pub first_line: String,
    pub lines: Vec<String>,
}
lazy_static! {
    static ref STATE_HEADER_INIT_REGEX: Regex =
        Regex::new(r"^(\d+):\s+<Initial predicate>$").unwrap();
}
impl TraceState {
    fn parse_init_header(s: &str) -> Option<usize> {
        let mut captures = STATE_HEADER_INIT_REGEX.captures_iter(s);
        if let Some(capture) = captures.next() {
            debug_assert!(captures.next().is_none());
            assert_eq!(capture.len(), 2);
            let idx = usize::from_str_radix(&capture[1], 10).expect("usize validated by regex");
            Some(idx)
        } else {
            None
        }
    }
}
lazy_static! {
    static ref STATE_HEADER_STATE_REGEX: Regex =
        Regex::new(r"^(\d+):\s+<Action line (\d+), col (\d+) to line (\d+), col (\d+) of module ([a-zA-Z_][a-zA-Z0-9_]*)>$").unwrap();
}
impl TraceState {
    fn parse_state_header(s: &str) -> Option<(usize, Span, String)> {
        let mut captures = STATE_HEADER_STATE_REGEX.captures_iter(s);
        if let Some(capture) = captures.next() {
            debug_assert!(captures.next().is_none());
            assert_eq!(capture.len(), 7);
            let idx = usize::from_str_radix(&capture[1], 10).expect("usize validated by regex");
            let start_row =
                usize::from_str_radix(&capture[2], 10).expect("usize validated by regex");
            let start_col =
                usize::from_str_radix(&capture[3], 10).expect("usize validated by regex");
            let end_row = usize::from_str_radix(&capture[4], 10).expect("usize validated by regex");
            let end_col = usize::from_str_radix(&capture[5], 10).expect("usize validated by regex");
            let span = Span::new((start_row, start_col), (end_row, end_col));
            let module = capture[6].to_string();
            Some((idx, span, module))
        } else {
            None
        }
    }
}
impl TraceState {
    /// Constructor.
    pub fn new(mut lines: Vec<String>) -> Res<Self> {
        if lines.is_empty() {
            bail!("trying to create a trace state from an empty description")
        }
        let first_line = lines.remove(0);

        if let Some(idx) = Self::parse_init_header(&first_line) {
            Ok(Self {
                idx,
                header: None,
                first_line,
                lines,
            })
        } else if let Some((idx, span, module)) = Self::parse_state_header(&first_line) {
            Ok(Self {
                idx,
                header: Some((span, module)),
                first_line,
                lines,
            })
        } else {
            Err(anyhow!("illegal first line for a state: `{}`", first_line))
                .context("failed to create trace state")
        }
    }
}

pub struct Span {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}
impl Span {
    pub fn new((start_row, start_col): (usize, usize), (end_row, end_col): (usize, usize)) -> Self {
        Self {
            start_row,
            start_col,
            end_row,
            end_col,
        }
    }
}
