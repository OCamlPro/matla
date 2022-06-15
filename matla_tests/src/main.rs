//! Top-level tests, require matla to be compiled in `debug` mode first.

use std::io::Write;

/// This crate's prelude.
#[macro_use]
pub mod prelude {
    pub use std::{
        fs,
        path::{Path, PathBuf},
    };

    pub use diff;
    pub use tempfile;

    pub use base::*;
    pub use conf;
    pub use project::{self, tlc::ConciseOutcome};

    pub use crate::run;

    /// Prints the result of `javac --version`.
    pub fn show_javac_version() {
        let output = io::Command::new("javac")
            .arg("--version")
            .output()
            .expect("failed to retrieve `javac` version with `javac --version`");
        let output = String::from_utf8_lossy(&output.stdout);
        println!("> javac --version\n{}", output)
    }

    /// Imports this crate's prelude.
    #[macro_export]
    macro_rules! prelude {
        ($($stuff:tt)*) => {
            use $crate::prelude::{*, $($stuff)*};
        };
    }
}

prelude!();

pub mod run;

pub struct Conf {
    parallel: bool,
    update: bool,
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            parallel: false,
            update: false,
        }
    }
}
impl Conf {
    pub fn new() -> Res<Self> {
        let mut args = std::env::args();
        let _ = args.next();

        let mut conf = Self::default();

        while let Some(arg) = args.next() {
            match &arg as &str {
                "--seq" | "--sequential" | "-s" => conf.parallel = false,
                "--par" | "--parallel" | "-p" => conf.parallel = true,
                "--update" | "-u" => conf.update = true,
                _ => bail!("unexpected argument `{}`", arg),
            }
        }
        Ok(conf)
    }
}

fn run(dirs: &[&str], get_conf: impl Fn() -> Option<Conf>) -> Res<()> {
    for dir in dirs {
        let conf = get_conf();
        let harness =
            Harness::new(conf, dir).with_context(|| anyhow!("building harness for `{}`", dir))?;
        harness
            .run_tests()
            .with_context(|| anyhow!("running tests in `{}`", dir))?
    }
    Ok(())
}
fn run_all(get_conf: impl Fn() -> Option<Conf>) -> Res<()> {
    run(&["tests/projects", "docs/manual/src"], get_conf)
}

#[test]
fn test() -> Res<()> {
    std::env::set_current_dir("..")
        .with_context(|| "failed to change current dir to parent directory")?;
    let get_conf = || {
        let mut conf = Conf::default();
        conf.parallel = false;
        Some(conf)
    };
    run_all(get_conf)
}

fn main() -> Res<()> {
    run_all(|| None)
}

pub struct Harness<'s> {
    dir: &'s str,
    conf: Conf,
    styles: conf::Styles,
    tests: Vec<run::Test>,
    inactive: Vec<run::Test>,
    matla_tla: String,
    matla_toml: String,
    /// Total number of **active** tests.
    ///
    /// # Invariant
    ///
    /// - `self.tests.len() = self.total`.
    total_active: usize,
}
impl<'s> Harness<'s> {
    pub fn new(conf: Option<Conf>, dir: &'s str) -> Res<Self> {
        let conf = if let Some(conf) = conf {
            conf
        } else {
            Conf::new().with_context(|| "failed handling command-line arguments")?
        };
        let styles = if atty::is(atty::Stream::Stdout) {
            conf::Styles::fancy()
        } else {
            conf::Styles::empty()
        };

        let matla_tla = {
            let mut buf: Vec<u8> = Vec::new();
            project::matla::write_module(&mut buf, false).expect("writing to String can't fail");
            String::from_utf8_lossy(&buf).into()
        };
        let matla_toml = {
            let mut buf: Vec<u8> = Vec::new();
            conf::Project::default()
                .ser_toml(&mut buf)
                .expect("writing to String can't fail");
            String::from_utf8_lossy(&buf).into()
        };

        // println!("scanning for tests in `{}`", tests_dir);
        let (tests, inactive) = run::Test::rec_load_from(dir)?;
        let total_active = tests.len();
        Ok(Self {
            dir,
            conf,
            styles,
            tests,
            inactive,
            total_active,
            matla_tla,
            matla_toml,
        })
    }

    fn run_tests(&self) -> Res<()> {
        let mut exit_with_2 = false;
        let test_plural = if self.total_active > 1 { "s" } else { "" };

        println!();
        let inactive_blah = if !self.inactive.is_empty() {
            format!(" ({} inactive)", self.inactive.len())
        } else {
            "".into()
        };

        println!(
            "running {} test{}{} from `{}`",
            self.styles.bold.paint(self.tests.len().to_string()),
            test_plural,
            self.styles.comment.paint(inactive_blah),
            self.styles.uline.paint(self.dir),
        );

        use base::rayon::prelude::*;
        let failed: usize = if self.conf.parallel {
            self.tests.par_iter().map(|test| self.handle(test)).sum()
        } else {
            self.tests.iter().map(|test| self.handle(test)).sum()
        };

        {
            // cleanup to avoid build files piling up and taking space
            let known_proj_dirs: sync::RwLock<Set<PathBuf>> = sync::RwLock::new(Set::new());
            let errors = sync::RwLock::new(vec![]);

            for test in &self.tests {
                let proj_path = test.proj_path();
                let known = {
                    let mut known = known_proj_dirs
                        .write()
                        .expect("[cleanup] lock on `known_proj_dirs` is poisoned");
                    if !known.contains(&proj_path) {
                        known.insert(proj_path.clone());
                        false
                    } else {
                        true
                    }
                };
                if !known {
                    match test.clean() {
                        Ok(()) => (),
                        Err(e) => errors
                            .write()
                            .expect("[cleanup] lock on `errors` is poisoned")
                            .push(e),
                    }
                }
            }

            let errors = errors
                .into_inner()
                .expect("[cleanup] final access to `errors` failed");
            let known_proj_dirs = known_proj_dirs
                .into_inner()
                .expect("[cleanup] final access to `known_proj_dirs` failed");

            if errors.is_empty() {
                println!(
                    "done cleaning {} test projects",
                    self.styles.bold.paint(known_proj_dirs.len().to_string())
                );
            } else {
                exit_with_2 = true;
                eprintln!(
                    "test project cleanup failed on {} project(s)",
                    self.styles.fatal.paint(errors.len().to_string())
                );
                for err in errors {
                    for (idx, line) in err.lines().enumerate() {
                        let pref = if idx == 0 { "- | " } else { "  | " };
                        eprintln!("{}{}", pref, line);
                    }
                }
            }
        }

        if failed > 0 {
            exit_with_2 = true;
            println!();
            debug_assert!(failed <= self.total_active);
            println!(
                "{} of {} tests {}",
                failed,
                self.total_active - failed,
                self.styles.bad.paint("failed"),
            );
        }

        if exit_with_2 {
            std::process::exit(2);
        }

        Ok(())
    }

    /// Checks that the matla `toml` and `tla` files are up to date in a project directory.
    fn check_matla_files(&self, proj_path: impl AsRef<Path>) -> Res<()> {
        let proj_path = proj_path.as_ref();

        let check = |file: &str, expected: &str| {
            let mut path = proj_path.to_path_buf();
            path.push(file);
            if path.is_file() {
                let content = io::load_file(&path)
                    .with_context(|| anyhow!("loading file `{}`", path.display()))?;
                if content != expected {
                    if self.conf.update {
                        let mut file = io::write_file(&path, true, false).with_context(|| {
                            anyhow!("failed to open writer for `{}`", path.display())
                        })?;
                        file.write(expected.as_bytes()).with_context(|| {
                            anyhow!("failed to update (write to) `{}`", path.display())
                        })?;
                    } else {
                        bail!("file `{}` is out of date", path.display());
                    }
                }
            }
            Ok(())
        };

        check(conf::project::TOML_CONFIG_FILENAME, &self.matla_toml)?;
        check("Matla.tla", &self.matla_tla)?;

        Ok(())
    }

    /// `0` if the test is successful, `1` otherwise.
    fn handle(&self, test: &run::Test) -> usize {
        // println!("- {}", test.name());
        // println!("  project dir: {}", test.proj_path().display());
        // println!("      command: matla {}", test.raw_cmd());
        // println!("         code: {}", test.code());
        // println!("  output:");
        // for line in test.output().lines() {
        //     println!("  | {}", line);
        // }

        // println!();
        // println!("running...");

        let constraints = test.contraints_desc();
        let constraints = constraints.as_ref().map(|s| s as &str).unwrap_or("");

        if !test.is_active() {
            println!(
                "- {}",
                self.styles.comment.paint(format!(
                    "{}{}: inactive by {}",
                    test.path_prefix().unwrap_or_else(|| "".into()),
                    test.name(),
                    constraints,
                ))
            );
            return 0;
        }

        let err = match self.check_matla_files(test.proj_path()) {
            Ok(()) => None,
            Err(e) => Some(e),
        };

        let mut res = test.run();
        if let Some(e) = err {
            res.add_error(e)
        }

        let test_pref = res.path_prefix().unwrap_or_else(|| "".into());
        let quals = {
            let quals = test.qualifiers();
            if quals.is_empty() {
                String::new()
            } else {
                let quals = quals.iter().fold(format!(""), |mut acc, qual| {
                    if !acc.is_empty() {
                        acc.push_str("|");
                    }
                    acc.push_str(&self.styles.uline.paint(qual).to_string());
                    acc
                });
                format!(" [{}]", quals)
            }
        };

        if let Some(report) = res.report(&self.styles, "  ") {
            // single `println!` to prevent another test's report from being injected between `println!`s
            println!(
                "- {}{}{}: {} {}\n{}",
                test_pref,
                self.styles.bold.paint(res.test.name()),
                quals,
                self.styles.fatal.paint("failed"),
                self.styles.comment.paint(constraints),
                report,
            );
            1
        } else {
            println!(
                "- {}{}{}: {} {}",
                test_pref,
                self.styles.bold.paint(res.test.name()),
                quals,
                self.styles.good.paint("success"),
                self.styles.comment.paint(constraints),
            );
            0
        }
    }
}
