//! TLC mode, just runs a TLC command.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// TLC subcommand name.
    const CMD_NAME: &str = "tlc";
    /// TLC options key.
    const ARGS_KEY: &str = "TLC_ARGS_KEY";

    /// TLC subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        let cmd = clap::Command::new(CMD_NAME).about("Calls TLC with some arguments.");
        crate::mode::run::cla::tlc_args(cmd).args(&[
            clap::Arg::new(ARGS_KEY)
                .help("Options to pass to the TLC command *directly*")
                .index(2)
                .last(true)
                .multiple_values(true)
                .value_name("TLC OPTIONS")
                .takes_value(true),
            clap::Arg::new(crate::mode::run::cla::SHOW_CONFIG_KEY)
                .help("Displays the options that matla will use to run TLC")
                .long("show_tlc_config"),
        ])
    }

    /// Constructs a [`Run`] if TLC subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|matches| {
            let opts = matches.values_of(ARGS_KEY);
            let args = opts
                .into_iter()
                .map(|s| {
                    s.into_iter()
                        .map(|s| s.split(char::is_whitespace))
                        .flatten()
                })
                .flatten()
                .filter_map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                })
                .collect();
            let (tlc_cla, module, release) = crate::mode::run::cla::handle_tlc_args(matches);
            let show_config = matches.is_present(crate::mode::run::cla::SHOW_CONFIG_KEY);
            Run::new(tlc_cla, module, release, args, show_config)
        })
    }
}

/// Runs TLC mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    /// TLC arguments.
    pub tlc_cla: conf::customl::TlcCla,
    /// Module to run.
    pub module: Option<String>,
    /// Build target.
    pub target: conf::Target,
    /// Trailing arguments for TLC.
    pub tail_args: Vec<String>,
    /// If true, show config before running.
    pub show_config: bool,
}
impl Run {
    /// Constructor.
    pub fn new(
        tlc_cla: conf::customl::TlcCla,
        module: Option<String>,
        release: bool,
        tail_args: Vec<String>,
        show_config: bool,
    ) -> Res<Self> {
        let target = conf::Target::new_run(conf::top_cla::project_path()?, release);
        Ok(Self {
            tlc_cla,
            module,
            target,
            tail_args,
            show_config,
        })
    }

    /// Launches a plain TLC command.
    pub fn launch(self) -> Res<io::ExitStatus> {
        // let mut cmd = conf::toolchain::tlc_cmd()?;
        // cmd.args(&self.args);

        let mut cmd = self
            .try_matla_project()?
            .expect("failed to generate TLC command");
        cmd.args(&self.tail_args);

        let (child, mut com) = thread::ChildCmd::new(cmd);
        let handle = child.spawn();
        let styles = conf::Styles::new();

        'handle_run: loop {
            match com.next() {
                None => {
                    log::debug!("child is done");
                    break;
                }
                Some(Ok((line, is_stderr))) => {
                    if is_stderr {
                        print!("{} ", styles.fatal.paint(">"));
                    }
                    println!("{}", line);
                    continue 'handle_run;
                }
                Some(Err(e)) => {
                    report_error(e, ": in TLC child");
                    continue 'handle_run;
                }
            }
        }

        match handle.join() {
            Ok(res) => res,
            Err(_) => bail!("failed to join TLC process"),
        }
    }

    /// Run TLC when inside a matla project.
    ///
    /// Involves
    /// - taking project configuration into account,
    /// - putting TLC's trash files in `target`.
    ///
    /// Returns `true` if successful.
    pub fn try_matla_project(&self) -> Res<Option<io::Command>> {
        let project_path = conf::top_cla::project_path()?;
        log::info!("loading project from `{}`", project_path.display());
        let project = project::SourceProject::from_path(&project_path)?;

        log::info!("creating actual build project");
        let (project, tlc_cla) = project.into_full(
            self.module.clone(),
            self.target.clone(),
            Some(&self.tlc_cla),
        )?;

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
        }

        let cmd = project.full_tlc_cmd(false)?;
        if self.show_config {
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
        Ok(Some(cmd))
    }
}
