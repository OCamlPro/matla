//! Uninstall mode.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Uninstall subcommand name.
    const CMD_NAME: &str = "uninstall";

    /// Uninstall subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Deletes your matla user directory (cannot be undone).")
            .args(&[crate::cla::top::project_path_arg().hide(true)])
    }

    /// Constructs a [`Run`] if uninstall subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|_| Run::new())
    }
}

/// Runs uninstall mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    /// Path to the user config directory.
    pub conf_dir: io::PathBuf,
}
impl Run {
    /// Constructor.
    pub fn new() -> Res<Self> {
        Ok(Self {
            conf_dir: conf::user::conf_path()?,
        })
    }

    /// Launches the uninstall mode.
    pub fn launch(&self) -> Res<()> {
        if !self.conf_dir.exists() {
            println!("No user configuration directory detected, nothing to do.");
            return Ok(());
        }

        println!(
            "You are about to delete your matla user directory `{}` recursively.",
            self.conf_dir.display()
        );
        let okay = io::ask_closed("", "Are you sure you want to proceed?", false)?;

        if okay {
            std::fs::remove_dir_all(&self.conf_dir).with_context(|| {
                format!("failed to remove directory `{}`", self.conf_dir.display())
            })?;
            println!("Successfully deleted `{}`.", self.conf_dir.display());
        } else {
            println!("Aborting uninstallation.")
        }

        Ok(())
    }
}
