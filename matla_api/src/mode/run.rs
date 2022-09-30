//! Run mode.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Run subcommand name.
    const CMD_NAME: &str = "run";
    /// Key for release mode.
    const RELEASE_KEY: &str = "RUN_RELEASE_KEY";
    /// Key for the module to run.
    const MAIN_MODULE_KEY: &str = "RUN_MAIN_MODULE_KEY";
    /// Key for showing configuration before running.
    pub const SHOW_CONFIG_KEY: &str = "RUN_SHOW_CONFIG_KEY";

    // TLC options.

    /// Key for the workers argument.
    const WORKERS_KEY: &str = "RUN_WORKERS_KEY";
    /// Workers default value.
    const WORKERS_DEFAULT: &str = "0";
    /// Key for the diff cexs.
    const DIFF_CEXS_KEY: &str = "RUN_DIFF_CEXS_KEY";
    /// Diff cexs default value.
    const DIFF_CEXS_DEFAULT: &str = "on";
    /// Key for the seed argument.
    const SEED_KEY: &str = "RUN_SEED_KEY";
    /// Seed default value.
    const SEED_DEFAULT: &str = "0";
    /// Key for the terse argument.
    const TERSE_KEY: &str = "RUN_TERSE_KEY";
    /// Terse default value.
    const TERSE_DEFAULT: &str = "off";
    /// Key for the max set size argument.
    const MAX_SET_SIZE_KEY: &str = "RUN_MAX_SET_SIZE_KEY";
    /// Max set size default value.
    const MAX_SET_SIZE_DEFAULT: &str = "default";
    /// Key for the check deadlocks argument.
    const CHECK_DEADLOCKS_KEY: &str = "RUN_CHECK_DEADLOCKS_KEY";
    /// Check deadlocks default value.
    const CHECK_DEADLOCKS_DEFAULT: &str = "on";
    /// Key for the print callstack argument.
    const PRINT_CALLSTACK_KEY: &str = "RUN_PRINT_CALLSTACK_KEY";
    /// Print callstack default value.
    const PRINT_CALLSTACK_DEFAULT: &str = "on";
    /// Key for the print timestats argument.
    const TIMESTATS_KEY: &str = "RUN_TIMESTATS_KEY";
    /// Print timestats default value.
    const TIMESTATS_DEFAULT: &str = "off";

    /// TLC arguments, separated for reusability.
    pub fn tlc_args(cmd: clap::Command<'static>) -> clap::Command {
        cmd.args([
            clap::Arg::new(RELEASE_KEY)
                .help("Activates release mode (deactivates debug checks in the Matla module)")
                .long_help(
                    "\
                    Release mode deactivates assertions in the `dbg` sub-module of \
                    the `Matla` module for efficiency. This only applies if `matla` \
                    generated a `Matla.tla` file in your project directory with \
                    `matla init`. Otherwise, debug/release modes are the same.\
                ",
                )
                .long("release"),
            crate::cla::top::project_path_arg(),
            // TLC CLA arguments.
            clap::Arg::new(WORKERS_KEY)
                .help("Sets the number of workers, `0` for `auto`")
                .long("workers")
                .short('w')
                .takes_value(true)
                .default_value(WORKERS_DEFAULT)
                .value_name(crate::cla::utils::val_name::NAT)
                .validator(|arg| crate::cla::utils::validate_usize_or_auto(&arg).map(|_| ())),
            clap::Arg::new(DIFF_CEXS_KEY)
                .help("If true, traces will only display state variables when they change")
                .long("diff_cexs")
                .takes_value(true)
                .default_value(DIFF_CEXS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|arg| crate::cla::utils::validate_bool(&arg).map(|_| ())),
            clap::Arg::new(SEED_KEY)
                .help("Sets the seed at TLC-level")
                .long("seed")
                .short('s')
                .takes_value(true)
                .default_value(SEED_DEFAULT)
                .value_name(crate::cla::utils::val_name::U64_OR_RANDOM)
                .validator(|s| crate::cla::utils::validate_u64_or_random(&s).map(|_| ())),
            clap::Arg::new(TERSE_KEY)
                .help("In terse mode, TLC will not execute print statements")
                .long("terse")
                .takes_value(true)
                .default_value(TERSE_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(MAX_SET_SIZE_KEY)
                .help("In terse mode, TLC will not execute print statements")
                .long("max_set_size")
                .takes_value(true)
                .default_value(MAX_SET_SIZE_DEFAULT)
                .value_name(crate::cla::utils::val_name::U64_OR_DEFAULT)
                .validator(|s| crate::cla::utils::validate_u64_or_default(&s).map(|_| ())),
            clap::Arg::new(CHECK_DEADLOCKS_KEY)
                .help(
                    "(De)activates deadlock checking; \
                    if `off`, TLC will not error on deadlocks",
                )
                .long("check_deadlocks")
                .takes_value(true)
                .default_value(CHECK_DEADLOCKS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(PRINT_CALLSTACK_KEY)
                .help(
                    "(De)activates callstack printing on errors; \
                    if `off`, TLC will not error on deadlocks",
                )
                .long("print_callstack")
                .takes_value(true)
                .default_value(PRINT_CALLSTACK_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(TIMESTATS_KEY)
                .help("(De)activates time statistics feedback")
                .long("timestats")
                .takes_value(true)
                .default_value(TIMESTATS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            // Done, there's just the optional module to run left.
            clap::Arg::new(MAIN_MODULE_KEY)
                .help(
                    "\
                    A legal module name to run TLC on, \
                    optional if only one module is *runnable* \
                    (has a `tla` and `cfg` file)\
                ",
                )
                .value_name(crate::cla::utils::val_name::MODULE)
                .index(1),
        ])
    }

    /// Run subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        let cmd = clap::Command::new(CMD_NAME)
            .about("Runs TLC on a TLA module in a project directory.")
            .args(&[clap::Arg::new(SHOW_CONFIG_KEY)
                .help("Displays the options that matla will use to run TLC")
                .long("show_tlc_config")]);
        tlc_args(cmd)
    }

    /// Extracts a [`conf::customl::TlcCla`], the main module and release flag from arguments from [`tlc_args`].
    pub fn handle_tlc_args(
        matches: &clap::ArgMatches,
    ) -> (conf::customl::TlcCla, Option<String>, bool) {
        macro_rules! retrieve {
            ( $key:ident with |$val:ident| $validator:expr ) => {
                if matches.occurrences_of($key) == 0 {
                    None
                } else {
                    let $val = matches.value_of($key).expect("argument with default value");
                    let res = $validator
                        .map_err(|e| Error::msg(e.to_string()))
                        .with_context(|| {
                            format!("value `{}` for `{}` was validated by clap", $val, $key,)
                        })
                        .expect("fatal error during CLAP");
                    Some(res)
                }
            };
        }
        let workers = retrieve!(
            WORKERS_KEY with
            |val| crate::cla::utils::validate_usize_or_auto(val)
        );
        let diff_cexs = retrieve!(
            DIFF_CEXS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let seed = retrieve!(
            SEED_KEY with
            |val| crate::cla::utils::validate_u64_or_random(val)
        );
        let terse = retrieve!(
            TERSE_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let max_set_size = retrieve!(
            MAX_SET_SIZE_KEY with
            |val| crate::cla::utils::validate_u64_or_default(val)
        );
        let check_deadlocks = retrieve!(
            CHECK_DEADLOCKS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let print_callstack = retrieve!(
            PRINT_CALLSTACK_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let timestats = retrieve!(
            TIMESTATS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let cla = conf::customl::TlcCla::new(
            conf::customl::Source::Cla,
            workers,
            diff_cexs,
            seed,
            terse,
            max_set_size,
            check_deadlocks,
            print_callstack,
            timestats,
        );

        let main_module = if let Some(main) = matches.value_of(MAIN_MODULE_KEY) {
            let main = if main.ends_with(".tla") {
                &main[..main.len() - 4]
            } else {
                main
            };
            Some(main.to_string())
        } else {
            None
        };

        let release = matches.is_present(RELEASE_KEY);

        (cla, main_module, release)
    }

    /// Constructs a [`Run`] if setup subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<mode::run::Run>> {
        matches.subcommand_matches(CMD_NAME).map(|matches| {
            let show_config = matches.is_present(SHOW_CONFIG_KEY);

            let (tlc_cla, main_module, release) = handle_tlc_args(matches);

            mode::run::Run::new(release, main_module, tlc_cla, show_config)
        })
    }
}

/// Runs setup mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    // /// TLC output log level.
    // pub tlc_log_level: log::LevelFilter,
    /// Target configuration.
    pub target: conf::Target,
    /// Main module: the module to run.
    pub main_module: Option<String>,
    /// Number of errors encountered.
    pub error_count: usize,
    /// User's options for the TLC command.
    pub tlc_cla: conf::customl::TlcCla,
    /// If true, display the options passed to TLC.
    pub show_config: bool,
}
impl Run {
    /// Constructor.
    ///
    /// Fails if `project_path` does not exist or is not a directory.
    pub fn new(
        // tlc_log_level: log::LevelFilter,
        release: bool,
        main_module: Option<String>,
        tlc_cla: conf::customl::TlcCla,
        show_config: bool,
    ) -> Res<Self> {
        let target = conf::Target::new_run(conf::top_cla::project_path()?, release);
        Ok(Self {
            // tlc_log_level,
            target,
            main_module,
            error_count: 0,
            tlc_cla,
            show_config,
        })
    }

    fn sorry_about_tlc() -> Error {
        let styles = conf::Styles::new();
        anyhow!(
            "Hey, it's {0}. Your project caused {1} to crash, which usually means that you\n\
            triggered a TLC-level error that {1} does not currently support. Sorry about that 😿\n\
            Note that you can run `{3}` instead of `{4}` to get the raw TLC\n\
            output.\n\
            \n\
            It would be great if you could open an issue on our repository:\n\
            - {2}\n\
            to notify us. Regardless, {5} for using {1}!",
            styles.ita.paint("Adrien"),
            styles.ita.paint("matla"),
            styles.good.paint("https://github.com/OCamlPro/matla"),
            styles.bold.paint("matla tlc ..."),
            styles.bold.paint("matla run ..."),
            styles.bold.paint("thank you")
        )
    }

    /// Launches a run.
    pub fn launch(self) -> Res<i32> {
        self.inner_launch()
    }
    fn inner_launch(self) -> Res<i32> {
        let project_path = conf::top_cla::project_path()?;
        log::info!("loading project from `{}`", project_path.display());
        let project = project::SourceProject::from_path(&project_path)?;

        log::info!("creating actual build project");
        let (project, tlc_cla) = project.into_full(
            self.main_module.clone(),
            self.target.clone(),
            Some(&self.tlc_cla),
        )?;
        log::debug!("- building to `{}`", project.target.path()?.display());

        if self.show_config {
            let mut bytes = Vec::<u8>::with_capacity(666);
            tlc_cla
                .ser_toml_source(&mut bytes, true)
                .context("failed to write TLC CLAs to bytes")?;
            let s = String::from_utf8_lossy(&bytes);
            println!("|===| TLC-level arguments");
            for line in s.lines() {
                println!("| {}", line);
            }
            println!("|===|");
            let cmd = project.full_tlc_cmd(true)?;
            print!("> {}", cmd.get_program().to_string_lossy());
            for arg in cmd.get_args() {
                print!(" \\\n    {}", arg.to_string_lossy());
            }
            println!();
            if let Some(path) = cmd.get_current_dir() {
                println!("| in `{}`", path.display());
            }
            println!()
        }

        log::info!("starting run on `{}`", project.actual_entry);
        let mut output_handler = TlcOutputHandler::new(log::LevelFilter::Warn, &project);
        let tlc = project
            .run_tlc_async(&mut output_handler)
            .with_context(|| {
                anyhow!("failed to launch TLC on module `{}`", project.actual_entry)
            })?;
        let outcome = tlc.run().with_context(Self::sorry_about_tlc)?;
        let concise = outcome.to_concise();

        let style = conf::Styles::new();

        let runtime = time::chrono_duration_fmt(&outcome.runtime);
        if project.tlc_cla.timestats {
            println!("done in {}", style.bold.paint(runtime));
        }

        use ConciseOutcome as Out;
        match concise {
            Out::Success => {
                vlog!(result | "specification is {}", style.good.paint("safe"));
            }
            Out::Unsafe => {
                vlog!(result | "specification is {}", style.fatal.paint("unsafe"));
            }
            Out::IllDefined => {
                vlog!(
                    result | "specification is {}",
                    style.bad.paint("ill-defined"),
                );
            }
            Out::Error(None) => {
                vlog!(
                    result | "specification caused an {}",
                    style.fatal.paint("error")
                );
            }
            Out::Error(Some(msg)) => {
                vlog!(
                    result | "specification caused an {}: {}",
                    style.fatal.paint("error"),
                    style.bad.paint(msg)
                );
            }
            Out::AssertFailed => {
                vlog!(
                    result | "specification {}",
                    style.fatal.paint("failed on an assertion")
                );
            }
            Out::Unknown => {
                bail!("failed to retrieve TLC process exit code or run outcome");
            }
        }

        Ok(concise.to_exit_code())
    }
}

#[derive(Debug, Clone)]
pub struct TlcOutputHandler<'a> {
    tlc_log_level: Option<log::Level>,
    start_instant: time::Instant,
    last_progress_update: time::Instant,
    progress_update_time_delta: time::Duration,
    style: conf::Styles,
    cexs: Vec<cex::Cex>,
    project: &'a project::FullProject,
}
impl<'a> TlcOutputHandler<'a> {
    /// True if there are counterexamples.
    pub fn has_cexs(&self) -> bool {
        !self.cexs.is_empty()
    }
}
impl<'a> TlcOutputHandler<'a> {
    /// Constructor.
    pub fn new(tlc_log_level: log::LevelFilter, project: &'a project::FullProject) -> Self {
        Self {
            tlc_log_level: tlc_log_level.to_level(),
            start_instant: time::Instant::now(),
            last_progress_update: time::Instant::now(),
            progress_update_time_delta: time::Duration::from_secs(1),
            style: conf::Styles::new(),
            cexs: vec![],
            project,
        }
    }

    /// True if some TLC-output log-level is active.
    pub fn is_log_active(&self, level: log::Level) -> bool {
        self.tlc_log_level.map(|l| level <= l).unwrap_or(false)
    }
}

mod msg_handling {
    use super::*;

    use project::tlc::code;

    impl<'a> TlcOutputHandler<'a> {
        /// Handles a [`code::Tlc`].
        pub fn handle_msg_status(&mut self, msg: &code::Status) {
            use code::Status;
            match msg {
                Status::TlcInitGenerated1 { state_count, .. } => {
                    vlog!(state stats|
                        "{}:\n  {}",
                        self.style.uline.paint("distinct initial state(s)"),
                        self.style.bold.paint(&state_count.1),
                    )
                }
                Status::TlcFinished { runtime } => {
                    vlog!(state stats|
                        "\n{} in {}",
                        self.style.uline.paint("done"),
                        self.style.bold.paint(time::duration_fmt(*runtime))
                    )
                }
                _ => (),
            }
        }

        /// Handles a [`code::Tlc`].
        pub fn handle_msg_tlc(&mut self, msg: &code::Tlc) {
            use code::Tlc;

            macro_rules! state_stats {
                (
                    // bool
                    last: $is_last:expr,
                    gene: ($gene:expr, $gene_spm:expr),
                    dist: ($dist:expr, $dist_spm:expr),
                    left: $left:expr $(,)?
                ) => {
                    let now = time::Instant::now();
                    let actually_output =
                        vlog!(if state stats { true } else { false })
                        && (
                            $is_last
                            || now - self.last_progress_update >= self.progress_update_time_delta
                        );
                    if actually_output {
                        self.last_progress_update = now;
                        use std::cmp::max;
                        let (gene, dist, left) = (&$gene.1, &$dist.1, &$left.1);
                        let (gene_spm, dist_spm): (&Option<(Int, String)>, &Option<(Int, String)>) =
                            ($gene_spm, $dist_spm);
                        let align = max(gene.len(), max(dist.len(), left.len()));
                        let gene_spm = if let Some((_, gen)) = gene_spm {
                            format!(", {} per minute", self.style.ita.paint(gen))
                        } else {
                            "".into()
                        };
                        let dist_spm = if let Some((_, dist)) = dist_spm {
                            format!(", {} per minute", self.style.ita.paint(dist))
                        } else {
                            "".into()
                        };
                        let runtime = vlog!(
                            if time stats {
                                let runtime = time::Instant::now() - self.start_instant;
                                format!("\n  runtime: {}", self.style.bold.paint(time::duration_fmt(runtime)))
                            } else {
                                "".into()
                            }
                        );
                        let header = if $is_last {
                            "final state stats"
                        } else {
                            "state stats"
                        };
                        vlog!(
                            state stats
                                | "{header}:{runtime}\n  \
                                {gene:>align$} generated{gene_spm}\n  \
                                {dist:>align$} {emph_dist}{dist_spm}\n  \
                                {left:>align$} left on queue\
                                ",
                                header = self.style.uline.paint(header),
                                gene = self.style.bold.paint(format!("{gene:>align$}")),
                                dist = self.style.bold.paint(format!("{dist:>align$}")),
                                emph_dist = self.style.good.paint("distinct"),
                                left = self.style.bold.paint(format!("{left:>align$}")),
                                align = align,
                        )
                    }
                }
            }

            match msg {
                Tlc::TlcSearchDepth { depth } => vlog!(
                    state stats | "{}:\n  {}",
                    self.style.uline.paint("search depth"),
                    self.style.bold.paint(depth.to_string()),
                ),
                Tlc::TlcStats {
                    generated,
                    distinct,
                    left,
                } => {
                    state_stats! {
                        last: true,
                        gene: (generated, &None),
                        dist: (distinct, &None),
                        left: left,
                    }
                }
                Tlc::TlcProgressStats {
                    generated,
                    gen_spm,
                    distinct,
                    dist_spm,
                    left,
                } => {
                    state_stats! {
                        last: false,
                        gene: (generated, gen_spm),
                        dist: (distinct, dist_spm),
                        left: left,
                    }
                }
                _ => (),
            }
        }
    }
}
impl<'a> project::tlc::Out for TlcOutputHandler<'a> {
    fn handle_outcome(&mut self, outcome: RunOutcome) {
        match outcome {
            RunOutcome::Success => (),
            RunOutcome::Failure(_) => (),
        }
    }
    fn handle_message(&mut self, msg: &project::tlc::msg::Msg, log_level: log::Level) {
        // Special handling for progress updates.
        use project::tlc::code::*;
        match msg.code.as_ref() {
            Some(TopMsg::Msg(Msg::Tlc(TlcMsg::Msg(tlc_msg)))) => self.handle_msg_tlc(tlc_msg),
            Some(TopMsg::Msg(Msg::Status(tlc_status))) => self.handle_msg_status(tlc_status),
            // Some(TopMsg::Msg(Msg::Tlc(TlcMsg::Msg(Tlc::TlcProgressStats {
            //     generated,
            //     gen_spm,
            //     distinct,
            //     dist_spm,
            //     left,
            // })))) => {
            //     let now = time::Instant::now();
            //     if self.last_progress_update - now >= self.progress_update_time_delta {
            //         println!(
            //             "generated {} states so far ({} distinct states)",
            //             pretty_usize(*generated),
            //             pretty_usize(*distinct),
            //         );

            //         // State-per-minute and states on queue update.
            //         if self.is_log_active(log::Level::Debug) {
            //             if gen_spm.is_some() || dist_spm.is_some() {
            //                 print!("↪ ");
            //                 let mut sep = "";
            //                 if let Some(gen) = *gen_spm {
            //                     print!("{} state per minute", pretty_usize(gen));
            //                     sep = ", ";
            //                 }
            //                 if let Some(dist) = *dist_spm {
            //                     print!(
            //                         "{}{} distinct state per minute",
            //                         sep,
            //                         pretty_usize(dist),
            //                     );
            //                 }
            //                 println!();
            //             }
            //             println!("{} states left on queue", left);
            //         }
            //     }
            //     return;
            // }
            Some(_) | None => (),
        }
        if self.is_log_active(log_level) {
            for line in msg.lines() {
                println!("{}", line);
            }
        }
    }

    fn handle_error(&mut self, error: impl Into<project::tlc::TlcError>) -> Res<()> {
        let error = error.into();
        let styles = conf::Styles::new();
        let pretty = error.pretty(self.project, &styles)?;

        if error.is_warning() {
            eprint!("{}: ", styles.uline.paint("Warning"));
        } else {
            eprint!("{}: ", styles.uline.paint("Error"));
        }
        for line in pretty {
            eprintln!("{}", line)
        }
        eprintln!();
        Ok(())
    }

    fn handle_cex(&mut self, cex: cex::Cex) {
        let mut buf = String::new();
        let spec = cex::pretty::Spec::default();
        spec.cex_to_ml_string(&cex, &mut buf);
        let (name_opt, is_temporal) = cex.falsified();
        match name_opt {
            Some(name) if is_temporal => println!(
                "Temporal property {} {}.",
                self.style.bad.paint(name),
                self.style.fatal.paint("does not hold"),
            ),
            Some(name) => println!(
                "Invariant {} {}.",
                self.style.bad.paint(name),
                self.style.fatal.paint("does not hold"),
            ),
            None if is_temporal => println!(
                "Some temporal property(ies) {}.",
                self.style.fatal.paint("do not hold"),
            ),
            None => println!(
                "Some invariant(s) {}.",
                self.style.fatal.paint("do not hold"),
            ),
        }
        println!(
            "{}",
            self.style
                .fatal
                .paint(self.style.uline.paint("Counterexample:").to_string())
        );
        for line in buf.lines() {
            println!("{}", line)
        }
        self.cexs.push(cex);
    }
}

#[cfg(feature = "with_clap")]
mod cla_spec {
    prelude!();

    /// Run subcommand name.
    const CMD_NAME: &str = "run";
    /// Key for release mode.
    const RELEASE_KEY: &str = "RUN_RELEASE_KEY";
    /// Key for the module to run.
    const MAIN_MODULE_KEY: &str = "RUN_MAIN_MODULE_KEY";
    // /// Key for TLC-level verbosity.
    // const TLC_VERB_KEY: &str = "RUN_TLC_VERB_KEY";
    /// Key for showing configuration before running.
    pub const SHOW_CONFIG_KEY: &str = "RUN_SHOW_CONFIG_KEY";

    // TLC options.

    /// Key for the workers argument.
    const WORKERS_KEY: &str = "RUN_WORKERS_KEY";
    /// Workers default value.
    const WORKERS_DEFAULT: &str = "0";
    /// Key for the diff cexs.
    const DIFF_CEXS_KEY: &str = "RUN_DIFF_CEXS_KEY";
    /// Diff cexs default value.
    const DIFF_CEXS_DEFAULT: &str = "on";
    /// Key for the seed argument.
    const SEED_KEY: &str = "RUN_SEED_KEY";
    /// Seed default value.
    const SEED_DEFAULT: &str = "0";
    /// Key for the terse argument.
    const TERSE_KEY: &str = "RUN_TERSE_KEY";
    /// Terse default value.
    const TERSE_DEFAULT: &str = "off";
    /// Key for the max set size argument.
    const MAX_SET_SIZE_KEY: &str = "RUN_MAX_SET_SIZE_KEY";
    /// Max set size default value.
    const MAX_SET_SIZE_DEFAULT: &str = "default";
    /// Key for the check deadlocks argument.
    const CHECK_DEADLOCKS_KEY: &str = "RUN_CHECK_DEADLOCKS_KEY";
    /// Check deadlocks default value.
    const CHECK_DEADLOCKS_DEFAULT: &str = "on";
    /// Key for the print callstack argument.
    const PRINT_CALLSTACK_KEY: &str = "RUN_PRINT_CALLSTACK_KEY";
    /// Print callstack default value.
    const PRINT_CALLSTACK_DEFAULT: &str = "on";
    /// Key for the print timestats argument.
    const TIMESTATS_KEY: &str = "RUN_TIMESTATS_KEY";
    /// Print timestats default value.
    const TIMESTATS_DEFAULT: &str = "off";

    /// TLC arguments, separated for reusability.
    pub fn tlc_args(cmd: clap::Command<'static>) -> clap::Command {
        cmd.args([
            clap::Arg::new(RELEASE_KEY)
                .help("Activates release mode (deactivates debug checks in the Matla module)")
                .long_help(
                    "\
                    Release mode deactivates assertions in the `dbg` sub-module of \
                    the `Matla` module for efficiency. This only applies if `matla` \
                    generated a `Matla.tla` file in your project directory with \
                    `matla init`. Otherwise, debug/release modes are the same.\
                ",
                )
                .long("release"),
            crate::cla::top::project_path_arg(),
            // TLC CLA arguments.
            clap::Arg::new(WORKERS_KEY)
                .help("Sets the number of workers, `0` for `auto`")
                .long("workers")
                .short('w')
                .takes_value(true)
                .default_value(WORKERS_DEFAULT)
                .value_name(crate::cla::utils::val_name::NAT)
                .validator(|arg| crate::cla::utils::validate_usize_or_auto(&arg).map(|_| ())),
            clap::Arg::new(DIFF_CEXS_KEY)
                .help("If true, traces will only display state variables when they change")
                .long("diff_cexs")
                .takes_value(true)
                .default_value(DIFF_CEXS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|arg| crate::cla::utils::validate_bool(&arg).map(|_| ())),
            clap::Arg::new(SEED_KEY)
                .help("Sets the seed at TLC-level")
                .long("seed")
                .short('s')
                .takes_value(true)
                .default_value(SEED_DEFAULT)
                .value_name(crate::cla::utils::val_name::U64_OR_RANDOM)
                .validator(|s| crate::cla::utils::validate_u64_or_random(&s).map(|_| ())),
            clap::Arg::new(TERSE_KEY)
                .help("In terse mode, TLC will not execute print statements")
                .long("terse")
                .takes_value(true)
                .default_value(TERSE_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(MAX_SET_SIZE_KEY)
                .help("In terse mode, TLC will not execute print statements")
                .long("max_set_size")
                .takes_value(true)
                .default_value(MAX_SET_SIZE_DEFAULT)
                .value_name(crate::cla::utils::val_name::U64_OR_DEFAULT)
                .validator(|s| crate::cla::utils::validate_u64_or_default(&s).map(|_| ())),
            clap::Arg::new(CHECK_DEADLOCKS_KEY)
                .help(
                    "(De)activates deadlock checking; \
                    if `off`, TLC will not error on deadlocks",
                )
                .long("check_deadlocks")
                .takes_value(true)
                .default_value(CHECK_DEADLOCKS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(PRINT_CALLSTACK_KEY)
                .help(
                    "(De)activates callstack printing on errors; \
                    if `off`, TLC will not error on deadlocks",
                )
                .long("print_callstack")
                .takes_value(true)
                .default_value(PRINT_CALLSTACK_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            clap::Arg::new(TIMESTATS_KEY)
                .help("(De)activates time statistics feedback")
                .long("timestats")
                .takes_value(true)
                .default_value(TIMESTATS_DEFAULT)
                .value_name(crate::cla::utils::val_name::BOOL)
                .validator(|s| crate::cla::utils::validate_bool(&s).map(|_| ())),
            // Done, there's just the optional module to run left.
            clap::Arg::new(MAIN_MODULE_KEY)
                .help(
                    "\
                    A legal module name to run TLC on, \
                    optional if only one module is *runnable* \
                    (has a `tla` and `cfg` file)\
                ",
                )
                .value_name(crate::cla::utils::val_name::MODULE)
                .index(1),
        ])
    }

    /// Extracts a [`conf::customl::TlcCla`], the main module and release flag from arguments from [`tlc_args`].
    pub fn handle_tlc_args(
        matches: &clap::ArgMatches,
    ) -> (conf::customl::TlcCla, Option<String>, bool) {
        macro_rules! retrieve {
            ( $key:ident with |$val:ident| $validator:expr ) => {
                if matches.occurrences_of($key) == 0 {
                    None
                } else {
                    let $val = matches.value_of($key).expect("argument with default value");
                    let res = $validator
                        .map_err(|e| Error::msg(e.to_string()))
                        .with_context(|| {
                            format!("value `{}` for `{}` was validated by clap", $val, $key,)
                        })
                        .expect("fatal error during CLAP");
                    Some(res)
                }
            };
        }
        let workers = retrieve!(
            WORKERS_KEY with
            |val| crate::cla::utils::validate_usize_or_auto(val)
        );
        let diff_cexs = retrieve!(
            DIFF_CEXS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let seed = retrieve!(
            SEED_KEY with
            |val| crate::cla::utils::validate_u64_or_random(val)
        );
        let terse = retrieve!(
            TERSE_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let max_set_size = retrieve!(
            MAX_SET_SIZE_KEY with
            |val| crate::cla::utils::validate_u64_or_default(val)
        );
        let check_deadlocks = retrieve!(
            CHECK_DEADLOCKS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let print_callstack = retrieve!(
            PRINT_CALLSTACK_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let timestats = retrieve!(
            TIMESTATS_KEY with
            |val| crate::cla::utils::validate_bool(val)
        );
        let cla = conf::customl::TlcCla::new(
            conf::customl::Source::Cla,
            workers,
            diff_cexs,
            seed,
            terse,
            max_set_size,
            check_deadlocks,
            print_callstack,
            timestats,
        );

        let main_module = if let Some(main) = matches.value_of(MAIN_MODULE_KEY) {
            let main = if main.ends_with(".tla") {
                &main[..main.len() - 4]
            } else {
                main
            };
            Some(main.to_string())
        } else {
            None
        };

        let release = matches.is_present(RELEASE_KEY);

        (cla, main_module, release)
    }

    impl mode::ClaMode for super::Run {
        const SUBCOMMAND_IDENT: &'static str = CMD_NAME;
        const PREREQ: mode::ClaModePrereq = mode::ClaModePrereq::Project;

        fn build_command(cmd: clap::Command<'static>) -> clap::Command<'static> {
            let cmd = cmd
                .about("Runs TLC on a TLA module in a project directory.")
                .args(&[
                    // clap::Arg::new(TLC_VERB_KEY)
                    //     .short('v')
                    //     .multiple_occurrences(true)
                    //     .help("Increases TLC output verbosity, capped at 3"),
                    clap::Arg::new(SHOW_CONFIG_KEY)
                        .help("Displays the options that matla will use to run TLC")
                        .long("show_tlc_config"),
                ]);
            tlc_args(cmd)
        }
        fn build(matches: &clap::ArgMatches) -> Res<Self> {
            // let tlc_log_level = match matches.occurrences_of(TLC_VERB_KEY) {
            //     0 => log::LevelFilter::Warn,
            //     1 => log::LevelFilter::Info,
            //     2 => log::LevelFilter::Debug,
            //     _ => log::LevelFilter::Trace,
            // };

            let show_config = matches.is_present(SHOW_CONFIG_KEY);

            let (tlc_cla, main_module, release) = handle_tlc_args(matches);

            Self::new(release, main_module, tlc_cla, show_config)
        }
        fn run(self) -> Res<Option<i32>> {
            self.launch().map(Some)
        }
    }
}
