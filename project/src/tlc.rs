//! Handles TLC runs.

prelude!();

pub mod code;
pub mod err;
pub mod msg;
pub mod outcome;
pub mod parse;
pub mod runtime;
pub mod warn;

pub use err::TlcError;

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
    fn handle_outcome(&mut self, outcome: RunOutcome);

    /// Handles an error.
    fn handle_error(&mut self, error: impl Into<tlc::err::TlcError>) -> Res<()>;

    /// Handles a cex.
    fn handle_cex(&mut self, cex: cex::Cex);
}
impl<'a, T: Out> Out for &'a mut T {
    fn handle_message(&mut self, msg: &msg::Msg, log_level: log::Level) {
        (*self).handle_message(msg, log_level)
    }
    fn handle_outcome(&mut self, outcome: RunOutcome) {
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
    fn handle_outcome(&mut self, _outcome: RunOutcome) {}
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

            let msg = if let Some(msg) = try_break!(self.tlc.next()) {
                msg
            } else {
                break 'doit;
            };
            let maybe_done = try_break!(self.runtime.handle(&mut self.out_handler, &msg));
            if let Some(nu_outcome) = maybe_done {
                self.out_handler.handle_outcome(nu_outcome.clone());
                outcome = Some(nu_outcome);
                break 'doit;
            }
        }
        let runtime = chrono::Utc::now() - start_time;
        {
            let error_count = self.runtime.tlc_error_fold(
                |cnt, err, reported| {
                    if !reported {
                        self.out_handler.handle_error(err)?;
                    }
                    Ok(cnt + 1)
                },
                0,
            )?;
            let overwrite =
                error_count > 0 && outcome.as_ref().map(|o| o.is_success()).unwrap_or(true);
            if overwrite {
                outcome = Some(RunOutcome::Failure(FailedOutcome::Plain(format!(
                    "{} error(s) occurred",
                    error_count,
                ))))
            }
        }
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
