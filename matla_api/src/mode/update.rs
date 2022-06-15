//! Update mode.
//!
//! Overwrites the TLA toolbox with the most recent release.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Update subcommand name.
    const CMD_NAME: &str = "update";

    /// Update subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Updates the `tla2tools` jar in the matla user directory.")
            .args(&[crate::cla::top::project_path_arg().hide(true)])
    }

    /// Constructs a [`Run`] if update subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|_| Run::new())
    }
}

/// Runs update mode.
#[derive(Debug, Clone)]
pub struct Run {
    setup: mode::setup::Run,
}
impl Run {
    /// Constructor.
    pub fn new() -> Res<Self> {
        mode::setup::Run::new(true, false, true).map(|setup| Self { setup })
    }

    /// Runs update mode.
    pub fn launch(&self) -> Res<()> {
        println!("Updating TLA toolbox...");
        let conf = match conf::user::try_read(|conf| conf.clone()) {
            Ok(conf) => conf,
            Err(e) => {
                report_error(e, ": problem retrieving user configuration file");
                println!();
                println!("Did you setup matla with `matla setup`?");
                println!("Are you sure matla is setup properly?");
                bail!("error loading configuration file")
            }
        };
        if conf.toolchain.tla2tools != conf::user::tla2tools_jar_path()? {
            println!("Matla is not setup in standalone mode.");
            println!(
                "It uses your local TLA toolbox `{}`, there's nothing to update",
                conf.toolchain.tla2tools.display()
            );
            println!("To setup matla in standalone mode, run `matla setup --standalone`");
            bail!("nothing to update, matla is not setup in standalone mode")
        }
        if !self.setup.tla2tools_jar_path.is_file() {
            bail!(
                "file `{}` does not exist, nothing to update",
                self.setup.tla2tools_jar_path.display()
            )
        }
        self.setup.update_toolbox()?;

        println!();
        println!("Updating process completed successfully.");
        Ok(())
    }
}
