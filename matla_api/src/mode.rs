//! Aggregates matla's run-modes.

pub mod clean;
pub mod init;
pub mod run;
pub mod setup;
pub mod testing;
pub mod tlc;
pub mod uninstall;
pub mod update;

#[cfg(feature = "with_clap")]
pub use self::requires_clap::*;

/// Aggregate all clap-related things in a private module.
#[cfg(feature = "with_clap")]
mod requires_clap {
    prelude!();

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ClaModePrereq {
        /// Mode runs before loading user config.
        PreUser,
        /// Mode runs after loading user config, but before loading project config.
        PreProject,
        /// Mode runs after loading user and project config.
        Project,
    }
    impl ClaModePrereq {
        pub fn is_pre_user(self) -> bool {
            self == Self::PreUser
        }
        pub fn is_pre_project(self) -> bool {
            self == Self::PreProject
        }
    }

    /// Trait implemented by modes.
    ///
    /// (Private) macro `enum_gather_specs` (manually) gathers all modes implementing this trait and
    /// builds the higher-level clap workflow. The macro can add flags for options that we want to
    /// appear both before and after the clap command, such as verbosity.
    pub trait ClaMode: Sized {
        const SUBCOMMAND_IDENT: &'static str;
        const PREREQ: ClaModePrereq;
        /// For error-reporting.
        const DESC: &'static str = Self::SUBCOMMAND_IDENT;
        fn build_command(cmd: clap::Command<'static>) -> clap::Command<'static>;
        fn build(matches: &clap::ArgMatches) -> Res<Self>;
        fn run(self) -> Res<Option<i32>>;
    }

    /// Generates the input enum and some helpers.
    ///
    /// # Input
    ///
    /// An enum definition with nullary variants. Variant identifiers are followed by `for
    /// $mode:ident` which must be the name of a mode-submodule with a `Run` type implementing
    /// [`ClaMode`] (and therefore [`ClaModeSpec`]).
    ///
    /// # Generates
    ///
    /// The enum `$ename` with variants `$variant(super::$mode::Run)`.
    ///
    /// - `list_all` returns a list of all variants of `$ename`;
    /// - `of_subcommand` returns the mode associated with a given subcommand;
    /// - `$ename` lifts all functions of [`ClaModeSpec`] from its variants.
    macro_rules! enum_gather_specs {
        (
            $(#[$emeta:meta])*
            $evis:vis enum $ename:ident {
                $(
                    $(#[$vmeta:meta])*
                    $variant:ident for $mode:ident
                ),*
                $(,)?
            }
        ) => (
            $(#[$emeta])*
            #[derive(Debug, Clone)]
            $evis enum $ename {
                $( $(#[$vmeta])* $variant(super::$mode::Run) , )*
            }

            impl $ename {
                pub fn prereq(&self) -> ClaModePrereq {
                    match self {
                        $(
                            Self::$variant(_) => super::$mode::Run::PREREQ,
                        )*
                    }
                }

                pub fn desc(&self) -> &'static str {
                    match self {
                        $(
                            Self::$variant(_) => super::$mode::Run::DESC,
                        )*
                    }
                }

                // pub fn build_command(self, cmd: clap::Command<'static>) -> clap::Command<'static> {
                //     match self {
                //         $(
                //             Self::$variant(_) => super::$mode::Run::build_command(cmd),
                //         )*
                //     }
                // }

                // pub fn add_subcommand(self, cmd : clap::Command<'static>) -> clap::Command<'static> {
                //     let mut sub = clap::Command::new(self.subcommand_ident());
                //     sub = self.build_command(sub);
                //     // add generic flags
                //     sub = cla::utils::sub_cmd::augment(sub);
                //     cmd.subcommand(sub)
                // }
                pub fn add_all_subcommands(mut cmd: clap::Command<'static>) -> clap::Command<'static> {
                    $(
                        let mut sub = clap::Command::new(super::$mode::Run::SUBCOMMAND_IDENT);
                        sub = super::$mode::Run::build_command(sub);
                        // add generic flags
                        sub = cla::utils::sub_cmd::augment(sub);
                        cmd = cmd.subcommand(sub);
                    )*
                    cmd
                }

                pub fn from_subcommand(matches: &clap::ArgMatches) -> Res<Self> {
                    let (sub, matches) = matches.subcommand().
                        ok_or_else(|| anyhow!("expected matla command, found nothing"))?;
                    $(
                        if sub == super::$mode::Run::SUBCOMMAND_IDENT {
                            // account for generic flags
                            cla::utils::sub_cmd::check_matches(matches)
                                .with_context(|| anyhow!("handling arguments for command `{}`", sub))?;
                            let mode = super::$mode::Run::build(matches)
                                .with_context(|| anyhow!("building `{}` mode", super::$mode::Run::DESC))?;
                            return Ok(Self::$variant(mode))
                        }
                    )*
                    bail!("unexpected command `{}`", sub)
                }

                pub fn run(self) -> Res<Option<i32>> {
                    match self {
                        $(
                            Self::$variant(mode) =>
                                mode.run().with_context(|| anyhow!(
                                    "failure while running `{}` mode", super::$mode::Run::DESC
                                )),
                        )*
                    }
                }
            }
        );
    }

    enum_gather_specs! {
        /// Gathers all modes.
        pub enum Mode {
            /// Project cleaning mode.
            Clean for clean,
            /// Project init mode.
            Init for init,
            /// Run mode.
            Run for run,
            /// Setup mode.
            Setup for setup,
            /// Test mode.
            Test for testing,
            /// TLC mode, only runs TLC.
            Tlc for tlc,
            /// Uninstals matla.
            Uninstall for uninstall,
            /// Updates the TLA+ toolchain.
            Update for update,
        }
    }
}
