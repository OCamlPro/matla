//! CLA handling.

/// CLA helpers.
pub mod utils {
    /// Static string for boolean arguments representing `true`.
    pub const BOOL_TRUE: &str = "on";
    /// Static string for boolean arguments representing `false`.
    pub const BOOL_FALSE: &str = "off";

    /// Value description for boolean arguments.
    pub const BOOL_VALUES: &str = "true|on|false|off";
    /// Validator for boolean arguments.
    pub fn validate_bool(arg: &str) -> Result<bool, String> {
        match arg {
            "true" | "on" => Ok(true),
            "false" | "off" => Ok(false),
            _ => Err(format!("expected {}", BOOL_VALUES)),
        }
    }

    pub fn validate_u64_or_random(s: &str) -> Result<Option<u64>, String> {
        match s {
            "random" | "Random" | "_" => Ok(None),
            _ => match u64::from_str_radix(s, 10) {
                Ok(res) => Ok(Some(res)),
                Err(e) => Err(e.to_string()),
            },
        }
    }
    pub fn validate_u64_or_default(s: &str) -> Result<Option<u64>, String> {
        match s {
            "default" | "Default" | "_" => Ok(None),
            _ => match u64::from_str_radix(s, 10) {
                Ok(res) => Ok(Some(res)),
                Err(e) => Err(e.to_string()),
            },
        }
    }
    pub fn validate_usize_or_auto(s: &str) -> Result<Option<usize>, String> {
        match s {
            "auto" | "Auto" | "_" => Ok(None),
            _ => match usize::from_str_radix(s, 10) {
                Ok(res) => Ok(Some(res)),
                Err(e) => Err(e.to_string()),
            },
        }
    }

    /// Value names, as displayed when asking clap for help.
    pub mod val_name {
        pub const DIR: &str = "DIR";
        pub const FILE: &str = "FILE";
        pub const MODULES: &str = "MODULES";
        pub const MODULE: &str = "MODULE";
        pub const BOOL: &str = super::BOOL_VALUES;
        pub const NAT: &str = "INT ≥ 0";

        pub const U64_OR_RANDOM: &str = "[Rr]andom|_|INT ≥ 0";
        pub const U64_OR_DEFAULT: &str = "[Dd]efault|_|INT ≥ 0";
    }

    pub fn if_flags_free_add(
        cmd: clap::Command<'static>,
        arg: impl FnOnce() -> clap::Arg<'static>,
        short: Option<char>,
        long: Option<&'static str>,
    ) -> (clap::Command<'static>, bool) {
        let (s, l) = cmd.get_arguments().fold((short, long), |(s, l), cmd| {
            let s = if s.is_some() && cmd.get_short() == s {
                None
            } else {
                s
            };
            let l = if l.is_some() && cmd.get_long() == l {
                None
            } else {
                l
            };
            (s, l)
        });
        if s.is_none() && l.is_none() {
            return (cmd, false);
        }

        let mut arg = arg();
        if let Some(s) = s {
            arg = arg.short(s);
        }
        if let Some(l) = l {
            arg = arg.long(l)
        }
        (cmd.arg(arg), true)
    }

    /// Handles teh color flag.
    pub mod color {
        prelude!();

        /// CLAP key for color.
        pub const COLOR_KEY: &str = "TOP_COLOR";
        pub const COLOR_KEY_LONG: &str = "color";
        pub const COLOR_KEY_SHORT: char = 'c';
        /// CLAP key for color.
        pub const COLOR_KEY_DEFAULT: &str = "on";

        pub fn add(cmd: clap::Command<'static>) -> clap::Command<'static> {
            let arg = || {
                clap::Arg::new(COLOR_KEY)
                    .short('c')
                    .long("color")
                    .help("(De)activates colored output.")
                    .takes_value(true)
                    .default_value(COLOR_KEY_DEFAULT)
                    .value_name(super::BOOL_VALUES)
                    .validator(|arg| super::validate_bool(&arg).map(|_| ()))
            };
            super::if_flags_free_add(cmd, arg, Some(COLOR_KEY_SHORT), Some(COLOR_KEY_LONG)).0
        }

        /// Extracts the internal log setting from the log flag value if explicitely provided.
        ///
        /// Produces `None` if no value was explicitely provided.
        pub fn try_explicit_of_matches(matches: &clap::ArgMatches) -> Option<bool> {
            use clap::ValueSource;
            if matches.try_contains_id(COLOR_KEY).is_err() {
                return None;
            }
            match matches.value_source(COLOR_KEY) {
                Some(ValueSource::CommandLine | ValueSource::EnvVariable) => {
                    let val = super::validate_bool(
                        matches
                            .value_of(COLOR_KEY)
                            .expect("unreachable: CLA value with default"),
                    )
                    .expect("unreachable: validated CLA value");
                    Some(val)
                }
                None | Some(ValueSource::DefaultValue) | Some(_) => None,
            }
        }
    }

    /// Handles internal [`base::log`] configuration.
    pub mod logger {
        pub const LOGGER_KEY: &str = "LOGGER_KEY";
        pub const LOGGER_KEY_LONG: &str = "log";
        pub const LOGGER_KEY_DEFAULT: &str = "warn";
        pub const LOGGER_KEY_DEFAULT_VAL: base::log::LevelFilter = base::log::LevelFilter::Warn;

        /// Adds internal log flag.
        pub fn add(cmd: clap::Command<'static>) -> clap::Command<'static> {
            let arg = || {
                let possible_values = [
                    clap::PossibleValue::new("warn"),
                    clap::PossibleValue::new("info"),
                    clap::PossibleValue::new("debug"),
                    clap::PossibleValue::new("trace"),
                ];
                clap::Arg::new(LOGGER_KEY)
                    .value_name("LOG_LEVEL")
                    .default_value(LOGGER_KEY_DEFAULT)
                    .value_parser(possible_values)
                    .help("Makes the internal logger more verbose, mostly for debugging.")
            };
            super::if_flags_free_add(cmd, arg, None, Some(LOGGER_KEY_LONG)).0
        }

        fn of_value(val: impl AsRef<str>) -> base::log::LevelFilter {
            use base::log::LevelFilter::*;
            match val.as_ref() {
                "warn" => Warn,
                "info" => Info,
                "debug" => Debug,
                "trace" => Trace,
                unexp => unreachable!(
                    "[clap] unexpected value `{}` for internal logger flag",
                    unexp,
                ),
            }
        }

        /// Extracts the internal log setting from the log flag value if explicitely provided.
        ///
        /// Produces `None` if no value was explicitely provided.
        pub fn try_explicit_of_matches(
            matches: &clap::ArgMatches,
        ) -> Option<base::log::LevelFilter> {
            use clap::ValueSource;
            if matches.try_contains_id(LOGGER_KEY).is_err() {
                return None;
            }
            match matches.value_source(LOGGER_KEY) {
                Some(ValueSource::CommandLine | ValueSource::EnvVariable) => {
                    matches.get_one::<String>(LOGGER_KEY).map(of_value)
                }
                None | Some(ValueSource::DefaultValue) | Some(_) => None,
            }
        }

        /// Extracts the internal log setting from the **top-level** log flag value.
        ///
        /// Unlike [`try_explicit_of_matches`], this function does not check that the `LOGGER_KEY`
        /// is an actual argument id. Should only be used on the top-level matches.
        pub fn of_top_matches(matches: &clap::ArgMatches) -> base::log::LevelFilter {
            let val = matches
                .get_one::<String>(LOGGER_KEY)
                .expect("arguments with default value always have a value");
            of_value(val)
        }
    }

    /// Handles user info verbosity.
    pub mod verb {
        pub const VERB_KEY: &str = "VERB_KEY";
        pub const VERB_KEY_SHORT: char = 'v';
        pub const QUIET_KEY: &str = "QUIET_KEY";
        pub const QUIET_KEY_SHORT: char = 'q';

        /// Adds internal flags.
        pub fn add(cmd: clap::Command<'static>) -> clap::Command<'static> {
            {
                let v = Some(VERB_KEY_SHORT);
                let q = Some(QUIET_KEY_SHORT);
                let dont_add = |arg: &clap::Arg| {
                    let s = arg.get_short();
                    s == v || s == q
                };
                if cmd.get_arguments().any(dont_add) {
                    return cmd;
                }
            }
            cmd.args([
                clap::Arg::new(VERB_KEY)
                    .short('v')
                    .action(clap::ArgAction::Count)
                    .help("Output more information such as statistics."),
                clap::Arg::new(QUIET_KEY)
                    .short('q')
                    .action(clap::ArgAction::Count)
                    .help("Output less information such as statistics."),
            ])
        }

        /// Extracts the number of *verb* and *quiet* flags.
        pub fn of_matches(matches: &clap::ArgMatches) -> (usize, usize) {
            let v = if matches.try_contains_id(VERB_KEY).is_ok() {
                matches.get_count(VERB_KEY)
            } else {
                0
            };
            let q = if matches.try_contains_id(QUIET_KEY).is_ok() {
                matches.get_count(QUIET_KEY)
            } else {
                0
            };
            (v as usize, q as usize)
        }
    }

    /// Add arguments and check matches for subcommands.
    pub mod sub_cmd {
        prelude!();

        pub fn augment(mut cmd: clap::Command<'static>) -> clap::Command<'static> {
            cmd = super::color::add(cmd);
            cmd = super::logger::add(cmd);
            cmd = super::verb::add(cmd);
            cmd
        }

        pub fn check_matches(matches: &clap::ArgMatches) -> Res<()> {
            if let Some(color) = super::color::try_explicit_of_matches(matches) {
                conf::top_cla::set_color(color)?
            }
            if let Some(level) = super::logger::try_explicit_of_matches(matches) {
                conf::top_cla::set_log_level(level).context("during CLAP")?
            }
            let (v, q) = super::verb::of_matches(matches);
            if v != 0 || q != 0 {
                conf::top_cla::verb_level_do(|mut level| {
                    level = level + v;
                    if q > level {
                        0
                    } else {
                        level - q
                    }
                })?
            }
            Ok(())
        }
    }
}

/// Matla's top-level CLAP.
#[cfg(feature = "with_clap")]
pub mod top {
    prelude!();

    // /// CLAP key for increasing verbosity.
    // pub const VERB_KEY: &str = "TOP_VERB";
    // /// CLAP key for decreasing verbosity.
    // pub const QUIET_KEY: &str = "TOP_QUIET";
    /// CLAP key for internal log verbosity.
    pub const LOG_KEY: &str = "TOP_LOG";
    /// CLAP key for portability.
    pub const PORTABLE_KEY: &str = "TOP_PORTABLE";
    /// CLAP key for project path.
    pub const PATH_KEY: &str = "TOP_PROJECT_PATH";
    /// Default key for project path.
    pub const PATH_KEY_DEFAULT: &str = ".";

    /// Adds the project path option.
    pub fn project_path_arg() -> clap::Arg<'static> {
        clap::Arg::new(PATH_KEY)
            .long("path")
            .takes_value(true)
            .default_value(PATH_KEY_DEFAULT)
            .value_name(cla::utils::val_name::DIR)
            .help("Path to a matla project directory")
    }
    /// Retrieves the project path from some matches, regardless of whether it is present.
    pub fn extract_project_path(matches: &clap::ArgMatches) -> io::PathBuf {
        matches
            .value_of(PATH_KEY)
            .expect("unreachable: CLA value with default")
            .into()
    }

    /// Sets the project path in [`conf::top_cla`].
    pub fn set_project_path(matches: &clap::ArgMatches) -> Res<()> {
        let path = extract_project_path(matches);
        conf::top_cla::set_project_path(&path)
    }
    /// Retrieves the project path from some matches, if present.
    pub fn get_project_path(matches: &clap::ArgMatches) -> Option<io::PathBuf> {
        // Check this first to make sure `PATH_KEY` is defined, other the code below will panic.
        // println!("- value_of");
        if matches.value_of(PATH_KEY).is_some() {
            // println!("- occurrences_of");
            if matches.occurrences_of(PATH_KEY) > 0 {
                // println!("- extract_project_path");
                return Some(extract_project_path(matches));
            }
        }
        None
    }
    /// Resolves the project path.
    ///
    /// Looks for a project path in `matches`, and then checks if there is a subcommand. If so,
    /// ignore the previous matches level's project path, recursively.
    pub fn resolve_project_path(matches: &clap::ArgMatches) -> io::PathBuf {
        let mut path = extract_project_path(matches);

        let mut sub_matches = matches.subcommand();
        while let Some((_sub_cmd, sub)) = sub_matches {
            // println!("sub_cmd: {:?}", _sub_cmd);
            if let Some(new_path) = get_project_path(sub) {
                path = new_path
            }
            sub_matches = sub.subcommand();
        }

        path
    }

    /// Top-level CLAP command.
    pub fn command(cmd: clap::Command<'static>) -> clap::Command<'static> {
        let cmd = super::utils::sub_cmd::augment(cmd);
        let cmd = cmd.args(&[
            // clap::Arg::new(VERB_KEY)
            //     .short('v')
            //     .multiple_occurrences(true)
            //     .help("increases verbosity"),
            // clap::Arg::new(QUIET_KEY)
            //     .short('q')
            //     .multiple_occurrences(true)
            //     .help("decreases verbosity"),
            clap::Arg::new(PORTABLE_KEY)
                .short('p')
                .long("portable")
                .help("Infer toolchain from environment, load no user configuration."),
            project_path_arg(),
        ]);
        mode::Mode::add_all_subcommands(cmd)
    }

    /// Performs top-level CLAP and returns the matches.
    pub fn init_from_matches(matches: &clap::ArgMatches) -> Res<()> {
        let portable = matches.is_present(cla::top::PORTABLE_KEY);

        let project_path: io::PathBuf = resolve_project_path(&matches);
        // actual color argument handled by `sub_cmd::check_matches` below
        let color = atty::is(atty::Stream::Stdout);

        conf::TopCla::new(color, portable, project_path).init()?;

        super::utils::sub_cmd::check_matches(matches)
    }
    pub fn init_from_str(cmd: clap::Command<'static>, clap: &'static str) -> Res<clap::ArgMatches> {
        let matches = command(cmd).get_matches_from(
            clap.split(|c: char| c.is_whitespace())
                .filter(|arg| !arg.is_empty()),
        );
        init_from_matches(&matches)?;
        Ok(matches)
    }
    pub fn init_from_env(cmd: clap::Command<'static>) -> Res<clap::ArgMatches> {
        let matches = command(cmd).get_matches();
        init_from_matches(&matches)?;
        Ok(matches)
    }
}
