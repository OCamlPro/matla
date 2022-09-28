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

    /// Handles internal [`base::log`] configuration.
    pub mod logger {
        pub const LOGGER_KEY: &str = "LOGGER_KEY";
        pub const LOGGER_KEY_DEFAULT: &str = "warn";

        /// Adds internal log flag.
        pub fn add(app: clap::Command<'static>) -> clap::Command<'static> {
            let possible_values = [
                clap::PossibleValue::new("warn"),
                clap::PossibleValue::new("info"),
                clap::PossibleValue::new("debug"),
                clap::PossibleValue::new("trace"),
            ];
            app.arg(
                clap::Arg::new(LOGGER_KEY)
                    .long("log")
                    .default_value(LOGGER_KEY_DEFAULT)
                    .value_parser(possible_values)
                    .help("makes the internal logger more verbose, mostly for debugging"),
            )
        }

        /// Extracts the internal log setting from the log flag value.
        pub fn of_matches(matches: &clap::ArgMatches) -> base::log::LevelFilter {
            use base::log::LevelFilter::*;
            let val = matches
                .get_one::<String>(LOGGER_KEY)
                .expect("argument with default are always present");
            match val as &str {
                "warn" => Warn,
                "info" => Info,
                "debug" => Debug,
                "trace" => Trace,
                _ => unreachable!("[clap] unexpected value for internal logger flag"),
            }
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
    /// CLAP key for color.
    pub const COLOR_KEY: &str = "TOP_COLOR";
    /// CLAP key for color.
    pub const COLOR_KEY_DEFAULT: &str = "on";
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
        let cmd = super::utils::logger::add(cmd);
        cmd.args(&[
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
                .help("infer toolchain from environment, load no user configuration"),
            project_path_arg(),
            clap::Arg::new(COLOR_KEY)
                .short('c')
                .long("color")
                .help("(de)activates colored output")
                .takes_value(true)
                .default_value(COLOR_KEY_DEFAULT)
                .value_name(cla::utils::BOOL_VALUES)
                .validator(|arg| cla::utils::validate_bool(&arg).map(|_| ())),
        ])
        .subcommands(mode::all_subcommands())
    }

    /// Performs top-level CLAP and returns the matches.
    pub fn init_from_matches(matches: &clap::ArgMatches) -> Res<()> {
        let log_level = super::utils::logger::of_matches(matches);
        let color = {
            let from_user = matches.is_present(cla::top::COLOR_KEY);
            let val = cla::utils::validate_bool(
                matches
                    .value_of(cla::top::COLOR_KEY)
                    .expect("unreachable: CLA value with default"),
            )
            .expect("unreachable: validated CLA value");

            if from_user {
                val
            } else {
                val && atty::is(atty::Stream::Stdout)
            }
        };
        let portable = matches.is_present(cla::top::PORTABLE_KEY);

        let project_path: io::PathBuf = resolve_project_path(&matches);

        conf::TopCla {
            portable,
            color,
            log_level,
            project_path,
        }
        .init()?;

        Ok(())
    }
    pub fn init_from_str(cmd: clap::Command<'static>, clap: &'static str) -> Res<clap::ArgMatches> {
        let matches = cla::top::command(cmd).get_matches_from(
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
