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

    /// Generates all the subcommands for all the modes.
    pub fn all_subcommands() -> [clap::Command<'static>; 8] {
        macro_rules! augmented_array {
            ( $($sub:expr),* $(,)? ) => (
                [$( cla::utils::sub_cmd::augment($sub), )*]
            );
        }
        augmented_array!(
            super::clean::cla::subcommand(),
            super::init::cla::subcommand(),
            super::run::cla::subcommand(),
            super::setup::cla::subcommand(),
            super::testing::cla::subcommand(),
            super::tlc::cla::subcommand(),
            super::uninstall::cla::subcommand(),
            super::update::cla::subcommand(),
        )
    }

    /// Wraps the result of `action()` in a `Some(_)`.
    macro_rules! wrap_try {
        ($e:expr) => {
            match $e {
                Ok(res) => res,
                Err(e) => return Some(Err(e.into())),
            }
        };
    }

    /// Given a clap module in `mode::`, calls function `cla::check_matches` of that module, and
    /// launches the result.
    macro_rules! check_launch_else_none {
        ( $matches:expr => ) => (
            return None
        );
        ( $matches:expr =>
            // name of the module
            $mod_name:ident
            // optional alias for the module, used for error reporting
            // if none, use `stringify!($mod_name)`
            $( ($name:expr) )?
            // optional custom result handler, expected to yield `Result<Option<i32>>` (exit code)
            $(
                where
                    |$res:ident| $action:expr
            )?
            $(, $($tail:tt)* )?
        ) => {
            if let Some(sub) = mode::$mod_name::cla::check_matches($matches) {
                #[allow(unused_assignments, unused_mut)]
                let mut desc =
                    stringify!($mod_name);
                $(
                    desc = $name as &str;
                )?
                let sub = wrap_try! {
                    sub.with_context(
                        || format!("`{}` mod initialization failed", desc)
                    )
                };
                #[allow(unused_variables)]
                let sub_res = wrap_try! {
                    sub.launch().with_context(
                        || format!("failure while running `{}` mode", desc)
                    )
                };
                #[allow(unused_assignments, unused_mut)]
                let mut real_res = Ok(None);
                $(
                    let $res = sub_res;
                    real_res = $action;
                )?
                return Some(real_res);
            }
            check_launch_else_none!($matches => $($($tail)*)?)
        };
    }

    /// Runs the mode specified by `matches`, if any.
    ///
    /// Only considers modes that must run **before** user configuration loading.
    pub fn try_pre_user_load(matches: &clap::ArgMatches) -> Option<Res<Option<i32>>> {
        check_launch_else_none!(matches =>
            setup,
            uninstall,
        );
    }

    /// Runs the mode specified by `matches`, if any.
    ///
    /// Only considers modes that must run **before** user configuration loading.
    pub fn try_pre_project_load(matches: &clap::ArgMatches) -> Option<Res<Option<i32>>> {
        check_launch_else_none!(matches =>
            init,
            update,
            tlc where
                |res| {
                    let code = wrap_try!(res.code().ok_or_else(
                        || anyhow!("failed to retrieve exit code of TLC process")
                    ));
                    Ok(Some(code))
                },
        );
    }

    /// Runs the mode specified by `matches`, if any.
    ///
    /// Only considers modes that must run **after** user configuration loading.
    pub fn try_post_loading(matches: &clap::ArgMatches) -> Option<Res<Option<i32>>> {
        check_launch_else_none!(matches =>
            run where
                |code| {
                    Ok(Some(code))
                },
            testing ("test"),
            clean,
        );
    }
}
