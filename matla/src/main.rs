//! Matla binary.

/// This crate's prelude.
pub mod prelude {
    pub use matla_api::prelude::*;
}
/// Imports this crate's prelude.
macro_rules! prelude {
    {$($stuff:tt)*} => {
        use $crate::prelude::{*, $($stuff)*};
    };
}

prelude!();

fn main() {
    let code = match inner_main() {
        Ok(None) => conf::exit_code::SAFE,
        Ok(Some(code)) => code,
        Err(e) => {
            for line in format!("Error: {:?}", e).lines() {
                eprintln!("{}", line)
            }
            conf::exit_code::ERROR
        }
    };
    std::process::exit(code)
}
fn inner_main() -> Res<Option<i32>> {
    let cmd = clap::Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!());
    let init = session::from_env_clas(cmd)?;

    // Set log-level.
    let log_level = conf::top_cla::log_level().context("retrieving cla log level")?;
    if let Some(level) = log_level.to_level() {
        simple_logger::init_with_level(level).context("during simple_logger init")?;
    }

    if let Some(res) = init.try_run() {
        return res;
    }

    let user_loaded = init.load_user_conf()?;
    if let Some(res) = user_loaded.try_run() {
        return res;
    }

    let project_loaded = user_loaded.load_project_conf()?;
    if let Some(res) = project_loaded.try_run() {
        return res;
    }

    Ok(None)
}
