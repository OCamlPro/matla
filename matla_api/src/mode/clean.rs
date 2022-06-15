//! Clean mode, just deletes the project's target directory.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Clean subcommand name.
    const CMD_NAME: &str = "clean";

    /// Clean subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Cleans the current project: deletes the `target` directory.")
            .args(&[crate::cla::top::project_path_arg()])
    }

    /// Constructs a [`Run`] if clean subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches
            .subcommand_matches(CMD_NAME)
            .map(|_matches| Run::new())
    }
}

/// Runs setup mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    /// Target directory configuration.
    pub target: conf::Target,
}
impl Run {
    /// Constructor.
    pub fn new() -> Res<Self> {
        let project_path = conf::top_cla::project_path()?;
        let target = conf::Target::new_run(&project_path, false);
        Ok(Self { target })
    }

    /// Launches the uninstall mode.
    pub fn launch(&self) -> Res<()> {
        if self.target.target_path.is_dir() {
            log::info!(
                "recursively deleting target directory `{}`",
                self.target.target_path.display()
            );
            io::remove_dir_all(&self.target.target_path).with_context(|| {
                anyhow!(
                    "failed to recursively delete target directory `{}`",
                    self.target.target_path.display()
                )
            })?;
        } else {
            log::warn!(
                "`{}` {}, nothing to clean",
                self.target.target_path.display(),
                if self.target.target_path.exists() {
                    "is not a directory"
                } else {
                    "does not exist"
                },
            );
        }

        Ok(())
    }
}
