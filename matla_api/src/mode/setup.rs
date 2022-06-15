//! Setup mode.
//!
//! Creates the user config file.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Setup subcommand name.
    const CMD_NAME: &str = "setup";
    /// Standalone mode key.
    const STANDALONE_KEY: &str = "SETUP_STANDALONE_KEY";
    /// From-env mode key.
    const FROM_ENV_KEY: &str = "SETUP_FROM_ENV_KEY";
    /// Overwrite mode key.
    const OVERWRITE_KEY: &str = "SETUP_OVERWRITE_KEY";

    /// Setup subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Performs this initial matla setup, required before running matla.")
            .args(&[
                clap::Arg::new(STANDALONE_KEY).long("standalone").short('s').help(
                    "Download the latest TLA toolbox to user directory and automatically use it",
                ),
                clap::Arg::new(FROM_ENV_KEY)
                    .long("from_env")
                    .help("Retrieve TLA toolbox path from the environment")
                    .conflicts_with(STANDALONE_KEY),
                clap::Arg::new(OVERWRITE_KEY)
                    .long("overwrite")
                    .short('o')
                    .help("Automatically overwrite config files when they exists"),
                crate::cla::top::project_path_arg().hide(true),
            ])
    }

    /// Constructs a [`Run`] if setup subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|matches| {
            let standalone = matches.is_present(STANDALONE_KEY);
            let from_env = matches.is_present(FROM_ENV_KEY);
            let overwrite = matches.is_present(OVERWRITE_KEY);
            Run::new(standalone, from_env, overwrite)
        })
    }
}

/// URL for the latest release of the TLA toolbox.
pub const TLA_TOOLBOX_URL: &str =
    "https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar";

/// Runs setup mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    /// Path to the configuration directory.
    pub conf_dir: io::PathBuf,
    /// Path to the toml configuration file.
    pub toml_path: io::PathBuf,
    /// Path to the tla2tools jar (only for standalone mode).
    pub tla2tools_jar_path: io::PathBuf,
    /// If true, download latest TLA toolbox and use that.
    pub standalone: bool,
    /// If true, retrieve TLA toolbox from environment.
    pub from_env: bool,
    /// If true, overwrite files when needed.
    pub overwrite: bool,
}
impl Run {
    /// Prefix for all lines in setup mode.
    pub const PREF: &'static str = "| ";

    /// Constructor.
    pub fn new(standalone: bool, from_env: bool, overwrite: bool) -> Res<Self> {
        if standalone && from_env {
            // Unreachable from setup CLAP as both arguments are exclusive.
            bail!(msg::fatal!(
                "trying to create setup mode in both standalone and from_env flag active"
            ))
        }
        Ok(Self {
            conf_dir: conf::user::conf_path()?,
            toml_path: conf::user::toml_path()?,
            tla2tools_jar_path: conf::user::tla2tools_jar_path()?,
            standalone,
            from_env,
            overwrite,
        })
    }
    /// Launches setup.
    pub fn launch(self) -> Res<()> {
        log::trace!(
            "running setup, config directory is `{}`",
            self.conf_dir.display()
        );
        println!("|===| TLA+ toolchain setup");
        self.inner_launch().context("matla setup failed")?;
        println!("|===|");
        Ok(())
    }
    /// Hidden launcher, used by [`Self::launch`] to factor error context augmentation.
    fn inner_launch(self) -> Res<()> {
        // Fail if config directory is a file.
        if self.conf_dir.is_file() {
            bail!(
                "configuration directory `{}` is a file, please delete it",
                self.conf_dir.display(),
            )
        }

        if !self.conf_dir.is_dir() {
            let okay = io::ask_closed(
                Self::PREF,
                format!(
                    "Config will live in `{}`, okay to create this directory?",
                    self.conf_dir.display(),
                ),
                true,
            )?;
            if !okay {
                println!(
                    "{}Aborting setup, you can still run matla in *portable* mode with `-p`",
                    Self::PREF,
                );
                return Ok(());
            }
        }

        io::create_dir_all(&self.conf_dir).with_context(|| {
            format!(
                "failed to create user configuration directory `{}`",
                self.conf_dir.display()
            )
        })?;

        if self.toml_path.is_dir() {
            bail!(
                "Configuration toml file `{}` is a directory, please delete it",
                self.toml_path.display(),
            )
        }
        if self.toml_path.is_file() {
            let okay = if self.overwrite {
                true
            } else {
                io::ask_closed(
                    Self::PREF,
                    format!(
                        "Config file `{}` already exists, ignore and overwrite?",
                        self.toml_path.display()
                    ),
                    false,
                )?
            };
            if !okay {
                println!(
                    "{}Aborting setup, keeping existing config file `{}`",
                    Self::PREF,
                    self.toml_path.display()
                );
                return Ok(());
            }
        }

        let toolchain =
            if let Some(conf) = self.toolchain().context("failed to setup TLA+ toolchain")? {
                conf
            } else {
                println!("{}Aborting toolchain setup.", Self::PREF);
                return Ok(());
            };

        println!("{}", Self::PREF);
        let conf = conf::Conf::new(toolchain);
        println!(
            "{}Writing configuration file to user directory, its content is:",
            Self::PREF,
        );
        println!("{}", Self::PREF);
        println!("{}```", Self::PREF);
        let mut bytes: Vec<u8> = Vec::with_capacity(666);
        conf.ser_toml(&mut bytes)?;
        for line in String::from_utf8_lossy(&bytes).lines() {
            let line = line.trim();
            if !line.is_empty() {
                println!("{}{}", Self::PREF, line)
            }
        }
        println!("{}```", Self::PREF);
        conf::user::dump(&conf, true).context("writing configuration to user directory")?;

        println!("{}", Self::PREF);
        println!(
            "{}Configuration regarding `tlc_cla` (TLC command-line arguments) corresponds to",
            Self::PREF,
        );
        println!(
            "{0:}options for `{1:} run`. You can check them out with `{1:} help run`.",
            Self::PREF,
            std::env::args()
                .into_iter()
                .next()
                .unwrap_or_else(|| "matla".into()),
        );
        println!(
            "{}The configuration above corresponds to matla's defaults, \
            and all items are optional.",
            Self::PREF,
        );
        println!("{}", Self::PREF);
        println!("{}Setup complete, matla is ready to go.", Self::PREF);

        Ok(())
    }

    /// Setup for the toolchain configuration.
    fn toolchain(&self) -> Res<Option<conf::Toolchain>> {
        let from_env = if self.from_env {
            true
        } else if self.standalone {
            false
        } else {
            println!("{}", Self::PREF);
            println!("{}Matla can either:", Self::PREF);
            println!(
                "{}- retrieve the tla2tools jar from your environment, or",
                Self::PREF,
            );
            println!("{}- download it for you.", Self::PREF);
            let download = io::ask_closed(
                Self::PREF,
                format!(
                    "Download the tla2tools to `{}`? \
                    If not, matla will attempt to find it in your path",
                    self.conf_dir.display(),
                ),
                true,
            )?;
            !download
        };

        let res = if from_env {
            self.toolchain_from_env()
                .context("failed to retrieve toolchain info from environment")
        } else {
            self.toolchain_download()
                .context("failed to download TLA toolchain")
        };
        res
    }

    /// Setup for the toolchain inferred from the environment.
    fn toolchain_from_env(&self) -> Res<Option<conf::Toolchain>> {
        let tla2tools_cmd = conf::toolchain::TLA2TOOLS_DEFAULT_NAME;
        match conf::Toolchain::from_env() {
            Ok(conf) => {
                println!(
                    "{}Success, TLA toolchain located at `{}`.",
                    Self::PREF,
                    conf.tla2tools.display()
                );
                return Ok(Some(conf));
            }
            Err(e) => {
                report_error(e, " retrieving TLA+ toolchain from environment:");

                println!("{}", Self::PREF);
                println!(
                    "{}Toolchain jar `{}` does not seem to be in your path.",
                    Self::PREF,
                    tla2tools_cmd,
                );
                let conf = io::ask(
                    Self::PREF,
                    format!(
                        "Please provide a file path or alternative command name for `{}`",
                        tla2tools_cmd,
                    ),
                    |path| {
                        self.toolchain_from_env_with(path)
                            .map_err(|e| e.to_string())
                    },
                )?;

                return Ok(Some(conf));
            }
        }
    }

    /// Toolchain config setup with a custom command.
    fn toolchain_from_env_with(&self, cmd: impl AsRef<io::Path>) -> Res<conf::Toolchain> {
        let cmd = io::PathBuf::from(cmd.as_ref());
        conf::Toolchain::from_env_with(&cmd).with_context(|| {
            format!(
                "failed to retrieve TLA toolchain from environment with `{}`",
                cmd.display(),
            )
        })
    }

    /// Toolchain config setup, standalone mode.
    ///
    /// Downloads the TLA toolbox to [`Self::tla2tools_jar_path`].
    fn toolchain_download(&self) -> Res<Option<conf::Toolchain>> {
        if self.tla2tools_jar_path.is_dir() {
            bail!(
                "toolchain download target `{}` is a directory, please delete it",
                self.tla2tools_jar_path.display()
            )
        }
        let force = if self.tla2tools_jar_path.is_file() && !self.overwrite {
            io::ask_closed(
                Self::PREF,
                format!(
                    "Toolchain download target `{}` already exists, overwrite?",
                    self.tla2tools_jar_path.display(),
                ),
                false,
            )?
        } else {
            true
        };

        if force {
            self.update_toolbox()?;
        } else {
            println!(
                "{}Skipping toolchain download, keeping existing one.",
                Self::PREF,
            )
        }

        self.toolchain_from_env_with(&self.tla2tools_jar_path)
            .map(Some)
    }

    /// Updates the tla2tool jar in the user directory.
    pub fn update_toolbox(&self) -> Res<()> {
        println!(
            "{}Downloading toolbox from `{}`...",
            Self::PREF,
            TLA_TOOLBOX_URL,
        );
        let body = io::download(TLA_TOOLBOX_URL)?
            .bytes()
            .with_context(|| format!("accessing the body of `{}`", TLA_TOOLBOX_URL))?;
        let body_byte_count = body.len();
        println!("{}Download completed successfully.", Self::PREF);

        println!(
            "{}Writing downloaded file to `{}`...",
            Self::PREF,
            self.tla2tools_jar_path.display()
        );
        let mut target = io::write_file(&self.tla2tools_jar_path, true, true)?;
        let bytes_written = {
            use io::Write;
            target.write(body.as_ref()).with_context(|| {
                format!(
                    "writing `{}` to `{}`",
                    TLA_TOOLBOX_URL,
                    self.tla2tools_jar_path.display(),
                )
            })?
        };
        if bytes_written != body_byte_count {
            bail!(
                "discrepancy writing `{}` to `{}`: wrote {} bytes of {}",
                TLA_TOOLBOX_URL,
                self.tla2tools_jar_path.display(),
                bytes_written,
                body_byte_count,
            )
        }

        Ok(())
    }
}
