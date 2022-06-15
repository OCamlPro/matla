//! Init mode.
//!
//! Sets up an existing directory with the basic layout of a matla project.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Init subcommand name.
    const CMD_NAME: &str = "init";
    /// Do not change/create a `.gitignore`.
    const NO_GITIGNORE_KEY: &str = "INIT_NO_GITIGNORE_KEY";
    /// Create project directory if needed.
    const NEW_KEY: &str = "INIT_NEW_KEY";
    /// Do not create `Matla.tla`.
    const NO_MATLA_KEY: &str = "INIT_NO_MATLA_KEY";
    /// Key for the path to the project directory.
    const PROJECT_PATH_KEY: &str = "RUN_PROJECT_PATH_KEY";
    /// Default project directory.
    const PROJECT_PATH_DEFAULT: &str = ".";

    /// Init subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Initializes an existing directory as a matla project.")
            .long_about(
                "By default, matla initializes an *existing* project directory by adding \
                the target (build) directory to the project's `.gitignore`, and creating \
                the `Matla` module.",
            )
            .args(&[
                clap::Arg::new(NO_GITIGNORE_KEY)
                    .long("no_gitignore")
                    .help("disables `.gitignore` creation/modification"),
                clap::Arg::new(NEW_KEY)
                    .long("new")
                    .help("create project directory if needed"),
                clap::Arg::new(NO_MATLA_KEY)
                    .long("no_matla_module")
                    .help("do not create a file for the `Matla` module"),
                clap::Arg::new(PROJECT_PATH_KEY)
                    .help("Path to the project directory to initialize")
                    .index(1)
                    .default_value(PROJECT_PATH_DEFAULT)
                    .value_name(crate::cla::utils::val_name::DIR),
                crate::cla::top::project_path_arg().hide(true),
            ])
    }

    /// Constructs a [`Run`] if init subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|matches| {
            let no_gitignore = matches.is_present(NO_GITIGNORE_KEY);
            let new = matches.is_present(NEW_KEY);
            let no_matla = matches.is_present(NO_MATLA_KEY);
            let project_path = io::PathBuf::from(
                matches
                    .value_of(PROJECT_PATH_KEY)
                    .expect("argument with default value"),
            );
            Run::new(project_path, no_gitignore, new, no_matla)
        })
    }
}

/// Runs init mode.
#[derive(Debug, Clone)]
pub struct Run {
    /// Create project directory if needed.
    pub new: bool,
    /// Deactivates `.gitignore` creation/modification.
    pub no_gitignore: bool,
    /// Deactivates Matla module creation.
    pub no_matla: bool,
    /// Path to the project directory.
    pub project_path: io::PathBuf,
}
impl Run {
    /// Constructor.
    pub fn new(
        project_path: impl Into<io::PathBuf>,
        no_gitignore: bool,
        new: bool,
        no_matla: bool,
    ) -> Res<Self> {
        Ok(Self {
            no_gitignore,
            new,
            no_matla,
            project_path: project_path.into(),
        })
    }

    /// Runs init mode.
    pub fn launch(&self) -> Res<()> {
        println!("Setting up your project, hang tight.");
        if !self.project_path.exists() {
            if self.new {
                io::create_dir_all(&self.project_path).with_context(|| {
                    anyhow!(
                        "failed to create project directory `{}`",
                        self.project_path.display()
                    )
                })?;
            } else {
                bail!(
                    "project directory `{}` does not exist, cannot initialize it",
                    self.project_path.display()
                );
            }
        }
        self.setup_gitignore()?;
        self.setup_matla_module()?;
        self.setup_toml_config_file()?;
        println!("Init complete, your project is ready to roll.");
        Ok(())
    }

    /// Creates the project configuration file.
    pub fn setup_toml_config_file(&self) -> Res<()> {
        println!("- setting up project configuration file...");

        let created = conf::Project::default().dump_to_dir(&self.project_path)?;
        if !created {
            println!(
                "  found a legal `{}` file, keeping it",
                conf::project::TOML_CONFIG_FILENAME
            );
        }

        Ok(())
    }

    /// Creates the Matla TLA module.
    ///
    /// Does nothing if `self.no_matla`.
    pub fn setup_matla_module(&self) -> Res<()> {
        if self.no_matla {
            println!("- skipping `Matla` module setup...");
            return Ok(());
        } else {
            println!("- setting up `Matla` module...");
        }

        let matla_module_path = {
            let mut path = self.project_path.clone();
            path.push("Matla.tla");
            path
        };

        if matla_module_path.is_file() {
            log::trace!("Matla module already exists, nothing to do");
        } else {
            log::trace!(
                "writing `{}` module (debug) to `{}`",
                project::matla::MATLA_MODULE_NAME,
                matla_module_path.display(),
            );
            let mut matla_module_file = io::write_file(&matla_module_path, false, false)?;
            project::matla::write_module(&mut matla_module_file, false).with_context(|| {
                anyhow!(
                    "failed to write `{}` module file `{}`",
                    matla_module_path.display(),
                    project::matla::MATLA_MODULE_NAME,
                )
            })?;
        }

        Ok(())
    }

    /// Creates/modifies `.gitignore`.
    ///
    /// Does nothing if `self.no_gitignore`.
    pub fn setup_gitignore(&self) -> Res<()> {
        use io::Write;

        if self.no_gitignore {
            println!("- skipping gitignore setup");
            return Ok(());
        } else {
            println!("- adding build directory to gitignore if needed...")
        }

        let gitignore_path = {
            let mut path = self.project_path.clone();
            path.push(".gitignore");
            path
        };
        let write_err = || anyhow!("failed to write to `{}`", gitignore_path.display());

        log::debug!(".gitignore setup `{}`", gitignore_path.display());

        let target_rule = format!("/{}", conf::target::TARGET_DIR_NAME);

        let mut gitignore_file = if gitignore_path.is_file() {
            log::trace!(".gitignore file already exists");
            let content = io::load_file(&gitignore_path)?;
            let is_target_rule = |s: &str| {
                let two_stars = format!("**/{}", conf::target::TARGET_DIR_NAME);
                s == target_rule || s == conf::target::TARGET_DIR_NAME || s == two_stars
            };
            // `target` already in `.gitignore`?
            if content.lines().any(is_target_rule) {
                log::trace!(
                    "`{}` already ignores target directory, nothing to do",
                    gitignore_path.display()
                );
                return Ok(());
            } else {
                log::trace!("appending 'ignore target directory' at the end of .gitignore");
                // Not there, append to `.gitignore`.
                let mut gitignore_file = io::write_file(&gitignore_path, true, false)?;
                // Used to known if we need to add a newline before the new rule.
                let mut last_line_empty = true;

                // Let's do this.
                for line in content.lines() {
                    last_line_empty = line.is_empty();
                    writeln!(&mut gitignore_file, "{}", line).with_context(write_err)?;
                }
                if !last_line_empty {
                    writeln!(&mut gitignore_file).with_context(write_err)?;
                }
                gitignore_file
            }
        } else {
            log::trace!("no .gitignore file, creating");
            // No `.gitignore` file, create it.
            io::write_file(&gitignore_path, false, false)?
        };

        writeln!(&mut gitignore_file, "# Ignore matla build directory.").with_context(write_err)?;
        writeln!(&mut gitignore_file, "{}", target_rule).with_context(write_err)?;

        Ok(())
    }
}
