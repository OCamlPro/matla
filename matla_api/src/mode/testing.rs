//! Test mode.

prelude!();

/// CLAP stuff.
#[cfg(feature = "with_clap")]
pub mod cla {
    use super::*;

    /// Test subcommand name.
    const CMD_NAME: &str = "test";
    /// Key for release mode.
    const RELEASE_KEY: &str = "TEST_RELEASE_KEY";
    /// Key for running tests in parallel.
    const RUN_PARALLEL_KEY: &str = "TEST_RUN_PARALLEL_KEY";
    /// Default value for running tests in parallel.
    const RUN_PARALLEL_DEFAULT: &str = crate::cla::utils::BOOL_FALSE;
    /// Key for the modules to run.
    const MAIN_MODULES_KEY: &str = "TEST_MAIN_MODULES_KEY";

    /// Test subcommand.
    pub fn subcommand() -> clap::Command<'static> {
        clap::Command::new(CMD_NAME)
            .about("Run the tests of a project.")
            .long_about(testing::integration::explain::test_conf())
            .args(&[
                crate::cla::top::project_path_arg(),
                clap::Arg::new(RUN_PARALLEL_KEY)
                    .help("(De)activates running tests concurrently")
                    .long("parallel")
                    .takes_value(true)
                    .value_name(crate::cla::utils::val_name::BOOL)
                    .default_value(RUN_PARALLEL_DEFAULT)
                    .validator(|arg| crate::cla::utils::validate_bool(&arg).map(|_| ())),
                clap::Arg::new(RELEASE_KEY)
                    .help("Activates release mode (deactivates debug checks in the Matla module)")
                    .long_help(
                        "\
                        Release mode deactivates assertions in the `dbg` sub-module of \
                        the `Matla` module for efficiency. This only applies if `matla` \
                        generated a `Matla.tla` file in your project directory with \
                        `matla init`. Otherwise, debug/release modes are the same.\
                    ",
                    )
                    .long("release"),
                clap::Arg::new(MAIN_MODULES_KEY)
                    .help(
                        "\
                            One ore more legal test module name(s) to run, \
                            if none then all tests will run\
                        ",
                    )
                    .value_name(crate::cla::utils::val_name::MODULES)
                    .takes_value(true)
                    .multiple_occurrences(true),
            ])
    }

    /// Constructs a [`Run`] if test subcommand is active.
    pub fn check_matches(matches: &clap::ArgMatches) -> Option<Res<Run>> {
        matches.subcommand_matches(CMD_NAME).map(|matches| {
            let release = matches.is_present(RELEASE_KEY);
            let parallel = {
                let arg = matches
                    .value_of(RUN_PARALLEL_KEY)
                    .expect("argument with default value");
                crate::cla::utils::validate_bool(arg)
                    .map_err(Error::msg)
                    .with_context(|| anyhow!("failed to parse argument despite validation"))?
            };
            let filter = if let Some(vals) = matches.values_of(MAIN_MODULES_KEY) {
                let mut filter = testing::Filter::new();
                for val in vals {
                    filter.add(val)?;
                }
                Some(filter)
            } else {
                None
            };
            Run::new(filter, release, parallel)
        })
    }
}

/// Runs setup mode.
#[readonly]
#[derive(Debug, Clone)]
pub struct Run {
    /// Source project.
    pub src_project: project::SourceProject,
    /// Test modules to run.
    pub filter: Option<testing::Filter>,
    /// True if in release mode.
    pub release: bool,
    /// True if running tests in parallel.
    pub parallel: bool,
    /// Path to the project directory.
    pub project_path: io::PathBuf,
}
impl Run {
    /// Constructor.
    pub fn new(filter: Option<testing::Filter>, release: bool, parallel: bool) -> Res<Self> {
        let project_path = conf::top_cla::project_path()?;
        let src_project = project::SourceProject::from_path(&project_path)?;
        Ok(Self {
            src_project,
            filter,
            release,
            parallel,
            project_path,
        })
    }

    /// Launches a plain TLC command.
    pub fn launch(self) -> Res<()> {
        let (passed, total) = self.integration()?;
        println!("integration tests: {} successful of {}", passed, total);
        if passed < total {
            bail!("{} integration test(s) failed", total - passed);
        } else if passed > total {
            panic!(
                "{} test(s) passed, but total number of tests is {}",
                passed, total
            );
        }
        Ok(())
    }

    /// Runs integration tests.
    ///
    /// Returns the number of tests passed and the total number of tests.
    pub fn integration(&self) -> Res<(usize, usize)> {
        let test_path = {
            let mut path = self.project_path.clone();
            path.push("tests");
            path
        };
        if !test_path.is_dir() {
            return Ok((0, 0));
        }
        let cxt = testing::integration::Cxt::dir_load(
            &test_path,
            &self.src_project,
            self.filter.as_ref(),
            self.release,
            true,
        )
        .context("failed to load integration tests")?;

        let styles = conf::Styles::new();

        let total = cxt.test_count();
        if total < 2 {
            println!("running {} integration test", total);
        } else if self.parallel {
            println!(
                "running {} integration tests {}",
                total,
                styles.bold.paint("concurrently")
            )
        } else {
            println!(
                "running {} integration tests {}",
                total,
                styles.bold.paint("sequentially")
            )
        }

        let res = cxt
            .run(self.parallel, |res, test| {
                let outcome = match &res {
                    Ok(Ok(())) => format!("{} 😺", styles.good.paint("success")),
                    Ok(Err(_)) => format!("{} 😿", styles.fatal.paint("failure")),
                    Err(_) => format!("{} 🙀", styles.bad.paint("unexpected error")),
                };
                println!(
                    "    test {}: {}",
                    styles.uline.paint(format!(
                        "`{}/{}`",
                        test.module_path_pref.display(),
                        test.module_name,
                    )),
                    outcome,
                );
                (res, test)
            })
            .context("failed to run integration tests")?;

        let mut passed = 0;
        let mut fatal_errors = false;
        for (res, test) in res {
            match res {
                Ok(sub) => match sub {
                    Ok(()) => passed += 1,
                    Err(lines) => {
                        println!();
                        log::error!("{}", lines[0]);
                        for line in lines[1..].iter() {
                            println!("{}", line);
                        }
                    }
                },
                Err(e) => {
                    fatal_errors = true;
                    println!();
                    log::error!(
                        "an unexpected error occurred on `{}`\n{:?}",
                        test.tla_path.display(),
                        e,
                    );
                }
            }
        }

        if fatal_errors {
            bail!(
                "some unexpected error(s) occurred, {} test{} passed of {}",
                passed,
                if passed > 1 { "s" } else { "" },
                total,
            )
        }

        Ok((passed, total))
    }
}
