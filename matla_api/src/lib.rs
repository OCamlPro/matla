//! Aggregates all the matla crates preludes.
//!
//! See
//! - [`base`] for ubiquitous dependencies, and helpers;
//! - [`conf`] for global configuration management, TLA command creation, and user configuration
//!   management;
//! - [`cex`] for counterexample (trace) handling;
//! - [`project`] for source/target project management;
//! - [`testing`] for test management.
//!
//! This crate has a `with_clap` feature (on by default) that activates CLAP-related features over
//! modes. It also gives access to the [`session`] module, which provides a session type encoding
//! matla's loading stages and what mode is legal to run when. Matla's binary crate is a shallow
//! wrapper around the session type.
//!
//! This library also provides [`mode`]s, which perform matla's various tasks: [`mode::run`],
//! [`mode::testing`], [`mode::setup`]...

#![forbid(rustdoc::broken_intra_doc_links)]

/// This crate's prelude.
pub mod prelude {
    #[cfg(feature = "with_clap")]
    pub use atty;
    #[cfg(feature = "with_clap")]
    pub use clap;

    pub use base::*;
    pub use cex;
    pub use conf;
    pub use project::{self, tlc::outcome::*};
    pub use testing;

    pub use crate::{cla, mode};

    // #[cfg(feature = "with_clap")]
    // pub use crate::session;

    /// Pretty-prints an error.
    pub fn report_error(e: Error, desc: impl fmt::Display) {
        log::error!("|=| Error{}", desc);
        for line in format!("{:?}", e).lines() {
            log::error!("  | {}", line);
        }
        log::error!("|=|")
    }
}

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
	( $($stuff:tt)* ) => (
		use $crate::prelude::{*, $($stuff)*};
	);
}

pub mod cla;
pub mod mode;

prelude!();

#[cfg(feature = "with_clap")]
pub use cla_api::*;

#[cfg(feature = "with_clap")]
mod cla_api {
    prelude!();

    pub fn mode_from_str_clas(cmd: clap::Command<'static>, clas: &'static str) -> Res<mode::Mode> {
        let matches = cla::top::init_from_str(cmd, clas)?;
        mode::Mode::from_subcommand(&matches)
    }

    pub fn mode_from_env_clas(cmd: clap::Command<'static>) -> Res<mode::Mode> {
        let matches = cla::top::init_from_env(cmd)?;
        mode::Mode::from_subcommand(&matches)
    }
}

/// Loads the user's configuration.
pub fn load_user_conf() -> Res<()> {
    match conf::user::load().with_context(|| anyhow!("failed to load configuration")) {
        Ok(()) => base::log::debug!("configuration loading successul"),
        Err(e) => {
            if !conf::top_cla::portable()? {
                bail!(e
                    .context("just re-run `matla setup` to make sure")
                    .context("if you have, your user directory might be corrupted")
                    .context("have you run `matla setup` yet?"))
            } else {
                bail!(e.context("are you sure the `tla2tools` java jar is in your path?"))
            }
        }
    }
    Ok(())
}

/// Loads the user's configuration, `true` iff project path exists.
pub fn load_project_conf() -> Res<bool> {
    conf::project::load()
}
