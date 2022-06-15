//! Handles the global configuration.
//!
//! This crate handles several configurations compartmentalized modules:
//! - [`top_cla`]: top-level options;
//! - [`user`]: global user configuration;
//! - [`project`]: project-level configuration;
//! - [`target`]: handles build paths.
//!
//! Feature-wise, this crate maintains a global [`Conf`] that other crates can use whenever they
//! need to retrieve configuration information, such as how to run TLC. The user configuration must
//! be loaded with [`load`][user::load] before it is accessed, otherwise trying to read it will
//! fail.
//!
//! Module [`user`] is in charge of producing user-configuration-related paths, such as the path to
//! the user's configuration file. It's main goal is to factor sub-paths, it is not expected to have
//! any use outside of this crate.

#![forbid(rustdoc::broken_intra_doc_links)]

pub extern crate toml;

#[macro_use]
pub mod prelude;

pub mod customl;
pub(crate) mod glob;
pub mod project;
pub mod target;
pub mod toolchain;
pub mod top_cla;
pub mod user;

pub use crate::prelude::*;

/// Matla exit codes.
pub mod exit_code {
    /// System is safe.
    pub const SAFE: i32 = 0;
    /// System is unsafe (cex-s).
    pub const UNSAFE: i32 = 10;
    /// System is ill-defined.
    pub const ILL_DEFINED: i32 = 20;
    /// System caused an assertion failure.
    pub const ASSERT_FAILED: i32 = 25;

    /// An error occurred.
    pub const ERROR: i32 = 2;

    /// No idea what happened, probably an internal/fatal error.
    pub const UNKNOWN: i32 = -1;

    /// Description of an exit code.
    pub fn desc(code: i32) -> base::Res<&'static str> {
        let desc = if code == SAFE {
            "safe"
        } else if code == UNSAFE {
            "unsafe"
        } else if code == ILL_DEFINED {
            "ill-defined"
        } else if code == ASSERT_FAILED {
            "assertion failure"
        } else if code == ERROR {
            "failure"
        } else if code == UNKNOWN {
            "unknown error"
        } else {
            return Err(base::anyhow!(
                "matla exit code `{}` does not exist and has no semantics",
                code,
            ));
        };
        Ok(desc)
    }
}

/// Performs a default test setup.
///
/// - sets up [`top_cla`].
pub fn default_test_setup() -> Res<()> {
    top_cla::register_test_version()?;
    Ok(())
}

/// True if the color flag is set.
pub fn color() -> Option<bool> {
    top_cla::read(|top| top.color)
}
/// Sets the color flag.
///
/// Only does something if the top-cla is set. If it is, returns `true`.
pub fn set_color(color: bool) -> bool {
    top_cla::write(|top| top.color = color).is_some()
}

/// Aggregates styling.
#[derive(Debug, Clone)]
pub struct Styles {
    pub bold: ansi::Style,
    pub ita: ansi::Style,
    pub uline: ansi::Style,
    pub good: ansi::Style,
    pub bad: ansi::Style,
    pub fatal: ansi::Style,
    pub comment: ansi::Style,
}
impl Styles {
    /// Constructor.
    pub fn new() -> Self {
        if color().unwrap_or(false) {
            Self::fancy()
        } else {
            Self::empty()
        }
    }

    /// Styleless contructor.
    pub fn empty() -> Self {
        Self {
            bold: ansi::Style::new(),
            ita: ansi::Style::new(),
            uline: ansi::Style::new(),
            good: ansi::Style::new(),
            bad: ansi::Style::new(),
            fatal: ansi::Style::new(),
            comment: ansi::Style::new(),
        }
    }
    /// Stylish contructor.
    pub fn fancy() -> Self {
        Self {
            bold: ansi::Style::new().bold(),
            ita: ansi::Style::new().italic(),
            uline: ansi::Style::new().underline(),
            good: ansi::Color::Green.bold(),
            bad: ansi::Color::Yellow.bold(),
            fatal: ansi::Color::Red.bold(),
            comment: ansi::Style::new().dimmed(),
        }
    }
}

/// Global configuration.
///
/// (De)serializable to (from) TOML.
#[derive(Debug, Clone)]
pub struct Conf {
    /// Toolchain configuration.
    pub toolchain: Toolchain,
}
impl Conf {
    /// Manual constructor.
    pub fn new(toolchain: Toolchain) -> Self {
        Self { toolchain }
    }
    /// Sets a configuration as the global one.
    ///
    /// Returns the previous configuration if any.
    pub fn register(self) -> Res<Option<Conf>> {
        let mut conf = glob::write()?;
        let old = mem::replace(&mut *conf, Some(self));
        Ok(old)
    }
}
impl Conf {
    /// Serializes itself to TOML.
    pub fn ser_toml(&self, w: &mut impl io::Write) -> Res<()> {
        self.toolchain.ser_toml(w)
    }
    /// Deserializes itself from TOML.
    pub fn de_toml(txt: &str) -> Res<Self> {
        let toolchain = Toolchain::de_toml(txt)?;
        Ok(Self { toolchain })
    }

    /// Attempts to build a toolchain configuration from the environment.
    pub fn from_env() -> Res<Self> {
        let toolchain = Toolchain::from_env()
            .context("failed to generate toolchain configuration from environment")?;
        Ok(Self { toolchain })
    }
}
implem! {
    for Conf {
        Display {
            |&self, fmt| {
                let mut bytes: Vec<u8> = Vec::with_capacity(666);
                self
                    .ser_toml(&mut bytes)
                    .map_err(|_| fmt::Error)
                    .expect("writing to String cannot fail");
                String::from_utf8_lossy(&bytes).fmt(fmt)
            }
        }
    }
}

/// TLC command-line options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlcCla {
    /// Number of workers to use, `0` or `None` for `auto`.
    pub workers: Option<usize>,
    /// Diff counterexamples.
    pub diff_cexs: bool,
    /// Run's seed, `None` for random..
    pub seed: Option<u64>,
    /// (De)activates TLC-level `Print` expressions: no printing if `true`.
    pub terse: bool,
    /// Maximum set size that TLC can enumerate, `None` for default.
    pub max_set_size: Option<u64>,
    /// (De)activates deadlock-checking.
    pub check_deadlocks: bool,
    /// (De)activates callstack-printing, *e.g.* for assertion failure reports.
    pub print_callstack: bool,
    /// (De)activates timestats-printing.
    pub timestats: bool,
}
impl TlcCla {
    /// Turns itself into a customl TLC command-line options.
    pub fn into_customl(self, source: customl::Source) -> customl::TlcCla {
        (self, source).into()
    }
}
implem! {
    for TlcCla {
        From<customl::TlcCla> {
            |toml| {
                let customl::TlcCla {
                    workers,
                    diff_cexs,
                    seed,
                    terse,
                    max_set_size,
                    check_deadlocks,
                    print_callstack,
                    timestats,
                } = toml;
                let mut slf = Self::default();
                workers.map(|(val, _)| slf.workers = val);
                diff_cexs.map(|(val, _)| slf.diff_cexs = val);
                seed.map(|(val, _)| slf.seed = val);
                terse.map(|(val, _)| slf.terse = val);
                max_set_size.map(|(val, _)| slf.max_set_size = val);
                check_deadlocks.map(|(val, _)| slf.check_deadlocks = val);
                print_callstack.map(|(val, _)| slf.print_callstack = val);
                timestats.map(|(val, _)| slf.timestats = val);
                slf
            }
        }
    }
}
impl Default for TlcCla {
    fn default() -> Self {
        Self {
            workers: Some(0),
            diff_cexs: true,
            seed: Some(0),
            terse: false,
            max_set_size: None,
            check_deadlocks: true,
            print_callstack: false,
            timestats: false,
        }
    }
}
impl TlcCla {
    /// Sets the number of workers, `0` for `auto`.
    pub fn workers(mut self, workers: Option<usize>) -> Self {
        self.workers = workers;
        self
    }
    /// Sets the [`Self::diff_cexs`] flag.
    pub fn diff_cexs(mut self, diff_cexs: bool) -> Self {
        self.diff_cexs = diff_cexs;
        self
    }
    /// Sets the seed, `None` for random.
    pub fn seed(mut self, seed: impl Into<Option<u64>>) -> Self {
        self.seed = seed.into();
        self
    }
    /// Sets the [`Self::terse`] flag.
    pub fn terse(mut self, terse: bool) -> Self {
        self.terse = terse;
        self
    }
    /// Sets the [`Self::max_set_size`] value, `None` for default.
    pub fn max_set_size(mut self, max_set_size: Option<u64>) -> Self {
        self.max_set_size = max_set_size;
        self
    }
    /// Sets the [`Self::check_deadlocks`] flag.
    pub fn check_deadlocks(mut self, check_deadlocks: bool) -> Self {
        self.check_deadlocks = check_deadlocks;
        self
    }

    /// Applies the arguments to an actual command.
    pub fn apply(&self, tlc_cmd: &mut io::Command) {
        tlc_cmd.arg("-workers");
        match self.workers {
            None | Some(0) => {
                tlc_cmd.arg("auto");
            }
            Some(w) => {
                tlc_cmd.arg(&w.to_string());
            }
        }
        if self.diff_cexs {
            tlc_cmd.arg("-difftrace");
        }
        if let Some(seed) = self.seed {
            tlc_cmd.args(["-seed", &seed.to_string()]);
        }
        if self.terse {
            tlc_cmd.arg("-terse");
        }
        if let Some(max_set_size) = self.max_set_size {
            tlc_cmd.args(["-maxSetSize", &max_set_size.to_string()]);
        }
        if !self.check_deadlocks {
            tlc_cmd.arg("-deadlock");
        }
    }

    /// [`Spawns`](io::Command::spawn) a TLC process.
    pub fn spawn(&self) -> Res<io::Child> {
        let mut cmd =
            toolchain::tlc_cmd().context("failed to retrieve TLC command from toolchain")?;
        self.apply(&mut cmd);
        cmd.spawn().map_err(Into::into)
    }
    /// Yields the [`Output`](io::Command::output) of a TLC process.
    pub fn output(&self) -> Res<io::Output> {
        let mut cmd =
            toolchain::tlc_cmd().context("failed to retrieve TLC command from toolchain")?;
        self.apply(&mut cmd);
        cmd.output().map_err(Into::into)
    }
}
