//! Deals with TOML-serde stuff.
//!
//! (De)serialization is custom so that matla can write comments: commented values and
//! documentation as comments.

prelude!();

pub mod parse;

/// A source for a TLC command-line argument: user, project, or command-line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// User configuration.
    User,
    /// Project configuration.
    Project,
    /// Command-line argument.
    Cla,
    /// Default value.
    Default,
    /// Custom source, for extensibility.
    Custom(&'static str),
    /// No source available.
    None,
}
impl Source {
    /// True if [`Self::None`].
    pub fn is_none(self) -> bool {
        match self {
            Self::None => true,
            Self::User | Self::Project | Self::Cla | Self::Custom(_) | Self::Default => false,
        }
    }
    /// True if [`Self::Default`].
    pub fn is_default(self) -> bool {
        match self {
            Self::Default => true,
            Self::User | Self::Project | Self::Cla | Self::Custom(_) | Self::None => false,
        }
    }
}
implem! {
    for Source {
        Display {
            |&self, fmt| match self {
                Self::User => {
                    write!(fmt,
                        "user configuration `{}`",
                        crate::user::conf_path()
                            .expect("failed to retrieve user configuration path")
                            .display()
                    )
                }
                Self::Project => "project configuration".fmt(fmt),
                Self::Cla => "command-line".fmt(fmt),
                Self::Default => "internal default configuration".fmt(fmt),
                Self::Custom(desc) => desc.fmt(fmt),
                Self::None => "<unknown origin>".fmt(fmt),
            }
        }
        From<&'static str> {
            |desc| Self::Custom(desc)
        }
    }
}

/// Same as [`TlcCla`] but geared towards (de)serialization to/from toml config files.
#[derive(Debug, Clone)]
pub struct TlcCla {
    /// Number of workers to use, `0` for `auto`.
    pub workers: Option<(Option<usize>, Source)>,
    /// Diff counterexamples.
    pub diff_cexs: Option<(bool, Source)>,
    /// Run's seed, `None` for random..
    pub seed: Option<(Option<u64>, Source)>,
    /// (De)activates TLC-level `Print` expressions: no printing if `true`.
    pub terse: Option<(bool, Source)>,
    /// Maximum set size that TLC can enumerate, `None` for default.
    pub max_set_size: Option<(Option<u64>, Source)>,
    /// (De)activates deadlock-checking.
    pub check_deadlocks: Option<(bool, Source)>,
    /// (De)activates callstack-printing, *e.g.* for assertion failure reports.
    pub print_callstack: Option<(bool, Source)>,
    /// (De)activates timestats-printing.
    pub timestats: Option<(bool, Source)>,
}
implem! {
    for TlcCla {
        From<(crate::TlcCla, Source)> {
            |(cla, source)| Self {
                workers: Some((cla.workers, source)),
                diff_cexs: Some((cla.diff_cexs, source)),
                seed: Some((cla.seed, source)),
                terse: Some((cla.terse, source)),
                max_set_size: Some((cla.max_set_size, source)),
                check_deadlocks: Some((cla.check_deadlocks, source)),
                print_callstack: Some((cla.print_callstack, source)),
                timestats: Some((cla.timestats, source)),
            }
        }
    }
}
impl Default for TlcCla {
    fn default() -> Self {
        (
            crate::TlcCla::default(),
            Source::from("internal default configuration"),
        )
            .into()
    }
}
impl TlcCla {
    /// Plain constructor.
    pub fn new(
        source: impl Into<Source>,
        workers: Option<Option<usize>>,
        diff_cexs: Option<bool>,
        seed: Option<Option<u64>>,
        terse: Option<bool>,
        max_set_size: Option<Option<u64>>,
        check_deadlocks: Option<bool>,
        print_callstack: Option<bool>,
        timestats: Option<bool>,
    ) -> Self {
        let source = source.into();
        Self {
            workers: workers.map(|val| (val, source)),
            diff_cexs: diff_cexs.map(|val| (val, source)),
            seed: seed.map(|val| (val, source)),
            terse: terse.map(|val| (val, source)),
            max_set_size: max_set_size.map(|val| (val, source)),
            check_deadlocks: check_deadlocks.map(|val| (val, source)),
            print_callstack: print_callstack.map(|val| (val, source)),
            timestats: timestats.map(|val| (val, source)),
        }
    }
    /// Constructor with all fields set to `None`.
    pub fn none() -> Self {
        Self {
            workers: None,
            diff_cexs: None,
            seed: None,
            terse: None,
            max_set_size: None,
            check_deadlocks: None,
            print_callstack: None,
            timestats: None,
        }
    }

    /// Pretty-prints itself in `toml` with explanations and commented default values.
    pub fn ser_toml_file(&self, w: &mut impl io::Write) -> Res<()> {
        macro_rules! doit {
            ($(
                $field:ident: $field_ty:literal ($quoted:expr) => {
                    $desc:expr,
                    Some($val:pat) => $some_blah:expr,
                }
            )*) => {{$(
                writeln!(w, "#\n# # {}.", $desc)?;
                let field_def_pref = concat!(stringify!($field), " =");
                match self.$field {
                    None => {
                        writeln!(w, concat!("# {} <", $field_ty, ">"), field_def_pref)
                            ?
                    }
                    Some(($val, _)) => {
                        let q = if $quoted { "'" } else { "" };
                        let inner = $some_blah;
                        write!(
                            w,
                            "# {} {}{}{}",
                            field_def_pref,
                            q,
                            inner,
                            q,
                        )?;

                        write!(w, concat!(" # <", $field_ty, ">"))?;
                    }
                }
            )*}};
        }

        // let default = TlcCla::default();

        writeln!(
            w,
            "# # Full configuration for TLC runtime arguments customization"
        )?;

        doit! {
            workers: "int|'auto'"(false) => {
                "Sets the number of workers, `0` or `auto` for `auto`",
                Some(w) => if let Some(w) = w { w.to_string() } else { "'auto'".to_string() },
            }
            diff_cexs: "'on'|'off'|'true'|'false'"(true) => {
                "If active, \
                counterexample traces will only display state variables when they change",
                Some(b) => if b { "on" } else { "off" },
            }
            seed: "int|'random'"(false) => {
                "Sets the seed when running TLC, random if none",
                Some(s) => match s {
                    Some(val) => val.to_string(),
                    None => "random".into(),
                },
            }
            terse: "'on'|'off'|'true'|'false'"(true) => {
                "If active, TLC will not output print statements",
                Some(b) => if b { "on" } else { "off" },
            }
            max_set_size: "u64|'default'"(true) => {
                "Maximum size of the sets TLC is allowed to enumerate",
                Some(max) => match max {
                    Some(max) => max.to_string(),
                    None => "default".into(),
                },
            }
            check_deadlocks: "'on'|'off'|'true'|'false'"(true) => {
                "If active, TLC will check for (and fail on) deadlocks",
                Some(b) => if b { "on" } else { "off" },
            }
            print_callstack: "'on'|'off'|'true'|'false'"(true) => {
                "If active, matla will present the callstack on errors, whenever possible",
                Some(b) => if b { "on" } else { "off" },
            }
            timestats: "'on'|'off'|'true'|'false'"(true) => {
                "If active, matla will present time statistics during runs",
                Some(b) => if b { "on" } else { "off" },
            }
        }

        Ok(())
    }
    /// Pretty-prints itself in `toml` with explanations and commented default values.
    pub fn ser_toml_source(&self, w: &mut impl io::Write, colored: bool) -> Res<()> {
        let styles = if colored {
            crate::Styles::new()
        } else {
            crate::Styles::empty()
        };
        let default = crate::TlcCla::default();
        macro_rules! doit {
            ($(
                $field:ident {
                    $desc:expr,
                    Some($val:pat) => $some_blah:expr,
                }
            )*) => {{$(
                let ($val, source) = self
                    .$field
                    .unwrap_or_else(|| (default.$field, Source::Default));
                writeln!(w, "{}", styles.comment.paint(format!("# {}.", $desc)))?;
                let inner = $some_blah;
                let (field, value) = (stringify!($field), inner.to_string());
                let (field_len, value_len) = (field.len(), value.len());
                let (field_off, value_off) = (
                    15.max(field_len) - field_len,
                    11.max(value_len) - value_len,
                );
                for _ in 0..field_off {
                    write!(w, " ")?;
                }
                write!(
                    w,
                    "{} = {}",
                    styles.bold.paint(stringify!($field)),
                    styles.good.paint(inner.to_string()),
                )?;

                if !source.is_none() {
                    for _ in 0..value_off {
                        write!(w, " ")?;
                    }
                    write!(w,
                        "{}",
                        styles.comment.paint(
                            format!(
                                " # from {}",
                                if source.is_default() {
                                    styles.bad.paint(source.to_string())
                                } else {
                                    styles.bold.paint(source.to_string())
                                }
                            )
                        )
                    )?;
                }
                writeln!(w)?;
            )*}};
        }

        doit! {
            workers {
                "Sets the number of workers, `0` for `auto`",
                Some(w) => if let Some(w) = w { w.to_string() } else { "'auto'".to_string() },
            }
            diff_cexs {
                "If active, \
                counterexample traces will only display state variables when they change",
                Some(b) => if b { "'on'" } else { "'off'" },
            }
            seed {
                "Sets the seed when running TLC, random if none",
                Some(s) => match s {
                    Some(val) => val.to_string(),
                    None => "'random'".into(),
                },
            }
            terse {
                "If active, TLC will not output print statements",
                Some(b) => if b { "'on'" } else { "'off'" },
            }
            max_set_size {
                "Maximum size of the sets TLC is allowed to enumerate",
                Some(max) => match max {
                    Some(max) => max.to_string(),
                    None => "'default'".into(),
                },
            }
            check_deadlocks {
                "If active, TLC will check for (and fail on) deadlocks",
                Some(b) => if b { "'on'" } else { "'off'" },
            }
            print_callstack {
                "If active, matla will present the callstack on errors, whenever possible",
                Some(b) => if b { "'on'" } else { "'off'" },
            }
            timestats {
                "If active, matla will present time statistics during runs",
                Some(b) => if b { "'on'" } else { "'off'" },
            }
        }

        Ok(())
    }

    /// Overwrites non-`None` values of `self` with those of `that`.
    pub fn receive(&mut self, that: &Self) {
        // If you're getting an error here, it probably means the fields of `Self` have changed and
        // this function needs to be updated.
        // Note that you need to update the destructuring let-binding below as well as the call to
        // the `overwrite!` macro several lines below.
        let TlcCla {
            workers,
            diff_cexs,
            seed,
            terse,
            max_set_size,
            check_deadlocks,
            print_callstack,
            timestats,
        } = that;
        macro_rules! overwrite {
            ( $($field:ident),* $(,)? ) => (
                $(
                    if $field.is_some() {
                        self.$field = $field.clone();
                    }
                )*
            );
        }
        overwrite!(
            workers,
            diff_cexs,
            seed,
            terse,
            max_set_size,
            check_deadlocks,
            print_callstack,
            timestats,
        );
    }
}
