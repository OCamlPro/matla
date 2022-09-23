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

    #[cfg(feature = "with_clap")]
    pub use crate::session;

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

/// Session types handling matla's top-level, requires the `with_clap` feature.
#[cfg(not(feature = "with_clap"))]
pub mod session {}

/// Session types handling matla's top-level.
#[cfg(feature = "with_clap")]
pub mod session {
    use super::*;
    prelude!();

    /// Constructor from custom CLAs.
    pub fn from_str_clas(cmd: clap::Command<'static>, clap: &'static str) -> Res<Init> {
        let matches = cla::top::init_from_str(cmd, clap)?;
        Ok(Init::new(matches))
    }
    /// Constructor from the environment's CLAs.
    pub fn from_env_clas(cmd: clap::Command<'static>) -> Res<Init> {
        let matches = cla::top::init_from_env(cmd)?;
        Ok(Init::new(matches))
    }

    /// Init top-level runner state.
    pub struct InitState;
    /// [`Runner`]'s init session type.
    pub type Init = Runner<InitState>;
    /// Runner state reached after loading the user configuration.
    pub struct UserLoadedState;
    /// [`Runner`]'s user-loaded session type.
    pub type UserLoaded = Runner<UserLoadedState>;
    /// Runner state reached after loading the project configuration.
    pub struct ProjectLoadedState;
    /// [`Runner`]'s project-loaded session type.
    pub type ProjectLoaded = Runner<ProjectLoadedState>;

    /// Matla top-level runner.
    pub struct Runner<State> {
        /// Matches.
        matches: clap::ArgMatches,
        /// Phantom state data.
        _phantom: std::marker::PhantomData<State>,
    }
    impl Init {
        /// Constructor from matches.
        fn new(matches: clap::ArgMatches) -> Self {
            Self {
                matches,
                _phantom: std::marker::PhantomData,
            }
        }
        /// Constructor from custom CLAs.
        pub fn from_str_clas(cmd: clap::Command<'static>, clap: &'static str) -> Res<Self> {
            let matches = cla::top::init_from_str(cmd, clap)?;
            Ok(Self::new(matches))
        }
        /// Constructor from the environment's CLAs.
        pub fn from_env_clas(cmd: clap::Command<'static>) -> Res<Self> {
            let matches = cla::top::init_from_env(cmd)?;
            Ok(Self::new(matches))
        }

        /// Log-level setter, used to bypass log-level from CLAP.
        pub fn set_log_level(self, log_level: log::LevelFilter) -> Res<Self> {
            conf::top_cla::set_log_level(log_level)?;
            Ok(self)
        }
        /// Color setter, used to bypass color flag from CLAP.
        pub fn set_color(self, color: bool) -> Res<Self> {
            conf::top_cla::set_color(color)?;
            Ok(self)
        }
        /// Log-level getter.
        pub fn log_level(self) -> Res<log::LevelFilter> {
            conf::top_cla::log_level()
        }

        /// Attempts to run modes that are legal at this stage.
        pub fn try_run(&self) -> Option<Res<Option<i32>>> {
            mode::try_pre_user_load(&self.matches)
        }

        /// Loads the user's configuration and transitions to the next session-state.
        pub fn load_user_conf(self) -> Res<UserLoaded> {
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
            Ok(Runner {
                matches: self.matches,
                _phantom: std::marker::PhantomData,
            })
        }
    }

    impl UserLoaded {
        /// Attempts to run modes that are legal at this stage.
        pub fn try_run(&self) -> Option<Res<Option<i32>>> {
            mode::try_pre_project_load(&self.matches)
        }

        /// Loads the user's configuration and transitions to the next session-state.
        pub fn load_project_conf(self) -> Res<ProjectLoaded> {
            conf::project::load()?;
            Ok(Runner {
                matches: self.matches,
                _phantom: std::marker::PhantomData,
            })
        }
    }

    impl ProjectLoaded {
        /// Attempts to run modes that are legal at this stage.
        pub fn try_run(&self) -> Option<Res<Option<i32>>> {
            mode::try_post_loading(&self.matches)
        }
    }
}
