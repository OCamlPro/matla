//! Target (project/build) directory configuration.
//!
//! A [`Target`] gathers the important paths for handling a project such as the path to the project
//! itself, the directory to build it in, the TLC meta-directory *etc.* There are three ways to
//! create a target:
//!
//! - [`Target::new_run`] for *run*/*build* builds;
//! - [`Target::new_test`] for *test* builds;
//! - [`Target::new_doc`] for *doc* builds.
//!
//! The first two have a notion of *debug*/*release* mode.

prelude!();

/// Name of the top target (build) directory.
pub const TARGET_DIR_NAME: &str = "target";

/// Stores the important directories needed for building a project.
#[readonly]
#[derive(Debug, Clone)]
pub struct Target {
    /// True if in release mode.
    pub release: bool,
    /// Path to the project directory.
    pub project_path: io::PathBuf,
    /// Path to the target directory.
    pub target_path: io::PathBuf,
    /// Actual build path.
    pub build_path: io::PathBuf,
    /// TLC metadir path.
    pub metadir_path: io::PathBuf,
}
impl Target {
    /// Constructor for a run target.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use conf::target::Target;
    /// use path_slash::PathExt;
    ///
    /// let project_path = "project/dir";
    /// let target = Target::new_run(project_path, true);
    /// assert_eq!(
    ///     target.project_path.to_slash_lossy(),
    ///     project_path,
    /// );
    /// assert_eq!(
    ///     target.target_path.to_slash_lossy(),
    ///     format!("{}/target", project_path),
    /// );
    /// assert_eq!(
    ///     target.build_path.to_slash_lossy(),
    ///     format!("{}/target/release", project_path),
    /// );
    /// assert_eq!(
    ///     target.metadir_path.to_slash_lossy(),
    ///     format!("{}/target/release/tlc_meta", project_path),
    /// );
    /// ```
    pub fn new_run(project_path: impl Into<io::PathBuf>, release: bool) -> Self {
        let project_path = project_path.into();
        let target_path = {
            let mut path = project_path.clone();
            path.push(TARGET_DIR_NAME);
            path
        };
        let build_path = {
            let mut path = target_path.clone();
            if release {
                path.push("release")
            } else {
                path.push("debug")
            }
            path
        };
        let metadir_path = {
            let mut path = build_path.clone();
            path.push("tlc_meta");
            path
        };
        Self {
            release,
            project_path,
            target_path,
            build_path,
            metadir_path,
        }
    }

    /// Builds a basic TLC command in `tool` mode with the `metadir` set.
    ///
    /// - `workers`: number of workers, `0` for `auto`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use conf::prelude::*;
    /// conf::default_test_setup();
    /// conf::user::load().expect("failed to load user configuration");
    ///
    /// let project_path = std::env::current_dir().expect("failed to retrieve current directory");
    /// let target = Target::new_run(&project_path, true);
    /// # println!(
    /// #     "project_path: `{}` (exists: {})", project_path.display(), project_path.exists()
    /// # );
    /// # println!(
    /// #     "build_path: `{}` (exists: {})",
    /// #     target.build_path.display(),
    /// #     target.build_path.exists(),
    /// # );
    /// std::fs::create_dir_all(&target.build_path).expect("failed to create build directory");
    /// let build_path = io::try_canonicalize(&target.build_path, true)
    ///     .unwrap()
    ///     .display()
    ///     .to_string();
    ///
    /// let tlc_cla = conf::TlcCla::default();
    /// let tlc_cmd = target.tlc_cmd(&tlc_cla).expect("unreachable");
    ///
    /// assert_eq!(
    ///     format!("{:?}", tlc_cmd),
    ///     format!(
    ///         "{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
    ///         conf::toolchain::tlc_cmd().expect("failed to retrieve TLC command from toolchain"),
    ///         "-tool",
    ///         "-metadir",
    ///         "tlc_meta",
    ///         "-workers",
    ///         "auto",
    ///         "-difftrace",
    ///         "-seed",
    ///         "0",
    ///     ),
    /// );
    /// ```
    pub fn tlc_cmd(&self, tlc_args: &crate::TlcCla) -> Res<io::Command> {
        self.custom_tlc_cmd(tlc_args, true)
    }
    pub fn custom_tlc_cmd(&self, tlc_args: &crate::TlcCla, tool: bool) -> Res<io::Command> {
        let mut cmd = toolchain::tlc_cmd()?;
        let cmd_working_dir = io::try_canonicalize(&self.build_path, false)?;
        cmd.current_dir(cmd_working_dir);
        // println!(
        //     "cmd current dir: `{}`",
        //     cmd.get_current_dir().unwrap().display(),
        // );
        if tool {
            cmd.arg("-tool");
        }
        let metadir = self.metadir_path.file_name().ok_or_else(|| {
            anyhow!(
                "failed to retrieve directory name of `{}`",
                self.metadir_path.display()
            )
        })?;
        cmd.arg("-metadir");
        cmd.arg(metadir);
        tlc_args.apply(&mut cmd);
        Ok(cmd)
    }

    /// Generates a TLC command with default [`TlcCla`].
    pub fn default_tlc_cmd(&self) -> Res<io::Command> {
        self.tlc_cmd(&crate::TlcCla::default())
    }

    /// Constructor for a test project.
    ///
    /// - `sub_dir`: name of the test sub-directory unique to the test this project is for.
    ///
    /// Very similar to [`Self::new_run`], but the build directory will be
    /// `<project_dir>/target/test/sub_dir`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use conf::target::Target;
    /// use path_slash::PathExt;
    ///
    /// let project_path = "project/dir";
    /// let sub_dir = "test_57";
    /// let target = Target::new_test(project_path, true, sub_dir);
    /// assert_eq!(
    ///     target.project_path.to_slash_lossy(),
    ///     project_path,
    /// );
    /// assert_eq!(
    ///     target.target_path.to_slash_lossy(),
    ///     format!("{}/target", project_path),
    /// );
    /// assert_eq!(
    ///     target.build_path.to_slash_lossy(),
    ///     format!("{}/target/release/tests/{}", project_path, sub_dir),
    /// );
    /// assert_eq!(
    ///     target.metadir_path.to_slash_lossy(),
    ///     format!("{}/target/release/tests/{}/tlc_meta", project_path, sub_dir),
    /// );
    /// ```
    pub fn new_test(
        project_path: impl Into<io::PathBuf>,
        release: bool,
        sub_dir: impl AsRef<str>,
    ) -> Self {
        let project_path = project_path.into();
        let target_path = {
            let mut path = project_path.clone();
            path.push(TARGET_DIR_NAME);
            path
        };
        let build_path = {
            let mut path = target_path.clone();
            if release {
                path.push("release")
            } else {
                path.push("debug")
            }
            path.push("tests");
            path.push(sub_dir.as_ref());
            path
        };
        let metadir_path = {
            let mut path = build_path.clone();
            path.push("tlc_meta");
            path
        };
        Self {
            release,
            project_path,
            target_path,
            build_path,
            metadir_path,
        }
    }

    /// Constructor.
    ///
    /// Very similar to [`Self::new_run`], but the build directory will be
    /// `<project_dir>/target/doc`. Also, documentation targets have no notion of debug/release.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use conf::target::Target;
    /// use path_slash::PathExt;
    ///
    /// let project_path = "project/dir";
    /// let target = Target::new_doc(project_path);
    /// assert_eq!(
    ///     target.project_path.to_slash_lossy(),
    ///     project_path,
    /// );
    /// assert_eq!(
    ///     target.target_path.to_slash_lossy(),
    ///     format!("{}/target", project_path),
    /// );
    /// assert_eq!(
    ///     target.build_path.to_slash_lossy(),
    ///     format!("{}/target/doc", project_path),
    /// );
    /// assert_eq!(
    ///     target.metadir_path.to_slash_lossy(),
    ///     format!("{}/target/doc/tlc_meta", project_path),
    /// );
    /// ```
    pub fn new_doc(project_path: impl Into<io::PathBuf>) -> Self {
        let project_path = project_path.into();
        let target_path = {
            let mut path = project_path.clone();
            path.push(TARGET_DIR_NAME);
            path
        };
        let build_path = {
            let mut path = target_path.clone();
            path.push("doc");
            path
        };
        let metadir_path = {
            let mut path = build_path.clone();
            path.push("tlc_meta");
            path
        };
        Self {
            release: false,
            project_path,
            target_path,
            build_path,
            metadir_path,
        }
    }
}
