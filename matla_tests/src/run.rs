//! Runs matla.

use std::io::{BufRead, Read};

use super::*;

/// Expected name of matla's binary as it appears in cargo's build directory.
#[cfg(target_os = "windows")]
const MATLA_BIN_FILENAME: &str = "matla.exe";
/// Expected name of matla's binary as it appears in cargo's build directory.
#[cfg(not(target_os = "windows"))]
const MATLA_BIN_FILENAME: &str = "matla";

/// Discriminates between OS-s.
#[derive(Debug, Clone, Copy)]
pub enum Os {
    Macos,
    Linux,
    Windows,
}
impl Os {
    /// True iff we are running on this OS.
    #[cfg(target_os = "macos")]
    pub fn eval(self) -> bool {
        match self {
            Self::Macos => true,
            Self::Linux | Self::Windows => false,
        }
    }
    /// True iff we are running on this OS.
    #[cfg(target_os = "linux")]
    pub fn eval(self) -> bool {
        match self {
            Self::Linux => true,
            Self::Macos | Self::Windows => false,
        }
    }
    /// True iff we are running on this OS.
    #[cfg(target_os = "windows")]
    pub fn eval(self) -> bool {
        match self {
            Self::Windows => true,
            Self::Macos | Self::Linux => false,
        }
    }

    /// Constructor.
    pub fn try_from(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "macos" => Some(Self::Macos),
            "linux" => Some(Self::Linux),
            "windows" => Some(Self::Windows),
            _ => None,
        }
    }

    /// String description.
    pub fn desc(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}
impl fmt::Display for Os {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}

/// Test constraints.
///
/// ~~Used in [`TestSpec`], constraints are encoded in qualifiers.~~
///
/// **Currently offline**: we need a way to distinguish qualifiers from constraints. Constraints
/// could be in the test file itself, but I like that you see the constraints just by looking at
/// the test file's name. Maybe have constraints start with `_` right after the separating `.`?
#[derive(Debug, Clone)]
pub enum TestConstraint {
    Os(Os),
    Not(Box<TestConstraint>),
}
impl TestConstraint {
    /// Negation prefix.
    pub const NEG_PREF: &'static str = "not_";

    /// Constructor (recursive).
    ///
    /// Should not stack overflow, this would require a super deeply negated constraint in the
    /// *name* of the test specification file.
    pub fn try_from(s: impl AsRef<str>) -> Option<Self> {
        let s = s.as_ref();

        if s.starts_with(Self::NEG_PREF) {
            Self::try_from(&s[Self::NEG_PREF.len()..]).map(Self::neg)
        } else if let Some(os) = Os::try_from(s) {
            Some(Self::Os(os))
        } else {
            None
        }
    }

    /// Negates itself.
    pub fn neg(self) -> Self {
        Self::Not(Box::new(self))
    }

    /// Constraint evaluation (recursive).
    ///
    /// Should not stack overflow, this would require a super deeply negated constraint in the
    /// *name* of the test specification file.
    pub fn eval(&self) -> bool {
        match self {
            Self::Os(os) => (*os).eval(),
            Self::Not(sub) => !sub.eval(),
        }
    }

    /// String description.
    pub fn desc(&self) -> String {
        match self {
            Self::Os(os) => os.desc().into(),
            Self::Not(sub) => format!("not({})", sub.desc()),
        }
    }
}
impl fmt::Display for TestConstraint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.desc().fmt(fmt)
    }
}

lazy_static! {
    /// Path to the matla binary.
    static ref MATLA_BIN: io::PathBuf = {
        let path = {
            let mut path = io::PathBuf::from("target/debug");
            path.push(MATLA_BIN_FILENAME);
            path
        };
        if !path.exists() {
            eprintln!("path to matla binary `{}` does not exist", path.display());
            let target = io::PathBuf::from("target/debug");
            if !target.exists() {
                eprintln!(
                    "path to rust's target directory `{}` does not exist",
                    target.display(),
                );
            } else if !target.is_dir() {
                eprintln!(
                    "path to rust's target directory `{}` is a file",
                    target.display(),
                );
            } else {
                eprintln!("contents of the target directory `{}`", target.display());
                for entry in target.read_dir().with_context(||
                    format!("failed to read the content of directory `{}`", target.display())
                ).expect("fatal error reading target directory") {
                    let entry = entry.with_context(
                        || format!("failed to read the content of directory `{}`", target.display(),)
                    ).expect("fatal error reading target directory");
                    eprintln!("- {}", entry.path().display());
                }
            }
            panic!("please run `cargo build` before running top-level tests")
        }
        let path = io::try_canonicalize(&path, true)
            .expect("failed to retrieve path to matla binary");
        path
    };
}

/// Test specification.
///
/// Corresponds to a `<PROJ_DIR> "." $(<TEST_QUAL> ".")* "test"` file with
///
/// - `<PROJ_DIR>`: *name* of the directory where the project to test is. Filesystem-wise, the
///   actualy project directory must be in the same directory as the `.test` file.
/// - `<TEST_QUAL>`: a list of strings.
#[derive(Debug, Clone)]
pub struct TestSpec {
    /// Path to the file itself.
    path: PathBuf,
    /// Test prefix, list of directories corresponding to the path to the test from the root.
    pref: Vec<String>,
    /// Name of the directory where the project to test is.
    proj_dir: String,
    // /// Test constraints, constructed from qualifiers.
    // constraints: Vec<TestConstraint>,
    qualifiers: Vec<String>,
}
impl TestSpec {
    /// Constructor.
    ///
    /// Returns `None` if `test_file_path` does not have extension `.test` or the path is not a
    /// directory. Does not check that the project directory actually exists. If it doesn't, failure
    /// will happen when we run the actual tests.
    ///
    /// # Panics
    ///
    /// - path does not exist;
    /// - file's stem is not accessible;
    /// - file's step is empty;
    /// - at least one test qualifier faild to yield a constraint.
    pub fn new(path: &Path, root: &Path) -> Option<Self> {
        if !path.exists() {
            panic!(
                "trying to load test from file `{}`, which does not exist",
                path.display()
            )
        }
        match (path.extension(), path.is_file()) {
            (Some(ext), true) if ext == "test" => (),
            _ => return None,
        }

        let pref = {
            let mut path = path.to_path_buf();
            let mut res = vec![];
            if let Some(parent) = path.parent() {
                path = parent.into()
            }

            loop {
                if path == root {
                    res.reverse();
                    break res;
                }
                if let Some(parent) = path.parent() {
                    if let Some(head) = path.file_name() {
                        res.push(head.to_string_lossy().to_string());
                        path = parent.into();
                    } else {
                        // failure, give up
                        break vec![];
                    }
                } else {
                    // `root` not found :/
                    break vec![];
                }
            }
        };

        let stem = if let Some(stem) = path.file_stem() {
            stem.to_string_lossy()
        } else {
            panic!("failed to retrieve stem of file `{}`", path.display())
        };

        let mut split = stem.as_ref().split('.').into_iter();

        let proj_dir = match split.next() {
            Some(dir) if !dir.is_empty() => dir.to_string(),
            _ => panic!(
                "failed to retrieve project directory from test file name `{}`",
                path.display()
            ),
        };

        let qualifiers = split.map(|s| s.to_string()).collect();
        // let constraints = {
        //     let mut constraints = vec![];
        //     for qual in split.map(|s| s.to_string()) {
        //         if let Some(c) = TestConstraint::try_from(&qual) {
        //             constraints.push(c);
        //         } else {
        //             panic!("unexpected test qualifier `{}`", qual)
        //         }
        //     }
        //     constraints
        // };

        Some(Self {
            path: path.into(),
            pref,
            proj_dir,
            qualifiers,
        })
    }

    /// Qualifier accessor.
    pub fn qualifiers(&self) -> &[String] {
        &self.qualifiers
    }

    /// True if the test is active, *i.e.* its constraints all evaluate to `true`.
    ///
    /// Constraints are currently not online, this always returns `true`.
    pub fn is_active(&self) -> bool {
        true
        // self.constraints.iter().all(TestConstraint::eval)
    }

    /// Path to the test specification file.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Path-refix with respect to the test root.
    pub fn path_prefix(&self) -> Option<String> {
        if self.pref.is_empty() {
            return None;
        }

        let mut s = self
            .pref
            .iter()
            .fold(String::with_capacity(37), |mut acc, pref| {
                if acc.is_empty() {
                    pref.to_string()
                } else {
                    acc.push_str("/");
                    acc.push_str(pref);
                    acc
                }
            });
        s.push_str("/");

        s.shrink_to_fit();
        Some(s)
    }
    /// Path to the project directory.
    pub fn proj_path(&self) -> PathBuf {
        let mut path = self.path.to_path_buf();
        path.set_file_name(&self.proj_dir);
        path
    }

    /// User-facing test name.
    pub fn name(&self) -> &str {
        &self.proj_dir
    }

    /// Constraints description, if any (recursive).
    ///
    /// Constraints are currently offline, this always returns `None`.
    pub fn contraints_desc(&self) -> Option<String> {
        None
        // let mut constraints = self.constraints.iter();

        // constraints.next().map(|first| {
        //     let mut s = constraints.fold(format!("cfg({})", first.desc()), |mut acc, next| {
        //         acc.push_str(", ");
        //         acc.push_str(&next.desc());
        //         acc
        //     });
        //     s.push(')');
        //     s
        // })
    }

    /// Same as [`Self::name`], but with the path to the test specification file.
    pub fn name_and_path(&self) -> String {
        let path = self.path.to_string_lossy();
        let path = path.as_ref();
        format!("{} ({})", self.name(), path)
    }
}

/// Context for a single test loaded from a test specification file.
///
/// For tests spec filename conventions, see [`TestSpec`].
///
/// The content of a test specification file is organized into the first line, the second line, and
/// the rest of the file.
///
/// - first line: `> matla $args`, specifies the matla command to run;
/// - second line: `# <isize>, specifies the exit code expected when running the matla command above;
/// - rest of the file: expected output, will be trimmed left and right.
#[derive(Debug, Clone)]
pub struct Test {
    /// Test specification file.
    spec: TestSpec,
    /// Matla command to run, **without** leading `matla`.
    cmd: String,
    /// Expected exit code.
    code: i32,
    /// Expected output.
    output: String,
}
impl std::ops::Deref for Test {
    type Target = TestSpec;
    fn deref(&self) -> &Self::Target {
        &self.spec
    }
}
impl Test {
    /// Recursively explores a directory, loading all the tests it finds, yields active and inactive
    /// tests separately.
    ///
    /// A test is a [`TestSpec`] file which specifies a project directory in the same directory as
    /// the test spec. This function does **not** recursively explore project directories. That is,
    /// tests nested into a test's project directory will be ignored.
    ///
    /// Probably should be an error actually.
    pub fn rec_load_from(from: impl AsRef<Path>) -> Res<(Vec<Self>, Vec<Self>)> {
        let root = from.as_ref();
        Self::inner_rec_load_from(root)
            .with_context(|| format!("loading tests from `{}`", root.display()))
    }

    fn inner_rec_load_from(from: impl AsRef<Path>) -> Res<(Vec<Self>, Vec<Self>)> {
        let root = from.as_ref().to_path_buf();
        let mut tests = Vec::with_capacity(100);
        let mut inactive = Vec::with_capacity(100);
        macro_rules! push {
            ($test:expr) => {
                if $test.is_active() {
                    tests.push($test)
                } else {
                    inactive.push($test)
                }
            };
        }

        // Set of directories to skip.
        let mut skip: Set<PathBuf> = Set::new();
        // Set of directories to explore.
        let mut todo: Set<PathBuf> = Set::new();

        let _is_new = todo.insert(root.clone());
        debug_assert!(_is_new);

        // Factor `skip`/`todo` operations, all of which expect a [`PathBuf`].
        macro_rules! dir_do {
            // Registers a test project directory: adds to `skip`, removes from `todo`.
            (register proj $dir:expr) => {
                let _was_there = todo.remove(&$dir);
                let _is_new = skip.insert($dir);
                // // can have several test specs for one proj dir
                // debug_assert!(_is_new);
            };
            // Registers a directory to explore later.
            (postpone $dir:expr) => {
                if !skip.contains(&$dir) {
                    let _is_new = todo.insert($dir);
                    debug_assert!(_is_new);
                }
            };
            (get next todo) => {
                if let Some(first) = todo.iter().next().cloned() {
                    todo.remove(first.as_path());
                    Some(first)
                } else {
                    None
                }
            };
        }

        // Extracts a directory to explore, handles test files, postpones sub-directories.
        //
        // We postpone all directories in `dir`: we're not exploring test project directories, so we
        // must handle test specs first to know to `skip` each project directory.
        '_explore: while let Some(dir) = dir_do!(get next todo) {
            'dir_entries: for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                // postpone all directories
                if path.is_dir() {
                    dir_do!(postpone path);
                    continue 'dir_entries;
                } else if !path.exists() {
                    bail!(
                        "exploration yielded path `{}` which does not exist",
                        path.display()
                    );
                }

                // `path` is guaranteed to be a file at this point
                if let Some(spec) = TestSpec::new(&path, &root) {
                    // `path` is a test specification
                    let test = Self::new(spec)?;
                    dir_do!(register proj test.spec.proj_path());
                    push!(test);
                } else {
                    // not a test specification file, skipping
                }
            }
        }

        tests.shrink_to_fit();
        inactive.shrink_to_fit();
        Ok((tests, inactive))
    }

    /// Loads a test specification.
    ///
    /// Note that `spec.proj_path()` must exist and be a directory.
    pub fn new(spec: TestSpec) -> Res<Self> {
        let proj_path = spec.proj_path();
        if !proj_path.exists() {
            bail!(
                "project directory `{}` for test specification file `{}` does not exist",
                proj_path.display(),
                spec.path().display(),
            );
        } else if !proj_path.is_dir() {
            bail!(
                "project directory `{}` for test specification file `{}` is not a directory",
                proj_path.display(),
                spec.path().display(),
            );
        }
        let path = spec.path.to_path_buf();
        Self::parse_spec(spec)
            .with_context(|| format!("on test specification file `{}`", path.display()))
    }

    /// Parses a test specification file.
    fn parse_spec(spec: TestSpec) -> Res<Self> {
        let mut file = io::BufReader::new(fs::OpenOptions::new().read(true).open(spec.path())?);
        let mut buf = String::with_capacity(666);
        macro_rules! abort {
            (msg) => {
                "ill-formed test specification"
            };
            () => {
                return Err(anyhow!(abort!(msg)))
            };
        }

        let cmd: String = {
            let bytes_read = file.read_line(&mut buf)?;
            if bytes_read == 0 {
                abort!();
            }
            let line = buf.trim();
            let start = "> matla";
            if line.starts_with(start) {
                line[start.len()..].trim().to_string()
            } else {
                abort!()
            }
        };
        buf.clear();

        let code: i32 = {
            let bytes_read = file.read_line(&mut buf)?;
            if bytes_read == 0 {
                abort!();
            }
            let line = buf.trim();
            let code = if line.starts_with("# ") {
                &line[2..]
            } else {
                abort!()
            };
            i32::from_str_radix(code, 10).with_context(|| abort!(msg))?
        };
        buf.clear();

        let output: String = {
            let _ = file.read_to_string(&mut buf)?;
            // if we need to trim left, just reallocate
            if buf.starts_with(char::is_whitespace) {
                buf.trim().to_string()
            } else {
                // otherwise just trim right without reallocation
                while buf.ends_with(char::is_whitespace) {
                    buf.pop();
                }
                buf.shrink_to_fit();
                buf
            }
        };

        Ok(Self {
            spec,
            cmd,
            code,
            output,
        })
    }

    /// Exit code accessor.
    pub fn code(&self) -> i32 {
        self.code
    }
    /// (Trimmed) output accessor.
    pub fn output(&self) -> &str {
        &self.output
    }
    /// Command accessor, string slice version.
    pub fn raw_cmd(&self) -> &str {
        &self.cmd
    }
    /// Builds the actual test command.
    pub fn cmd(&self) -> duct::Expression {
        let args =
            ["--color", "off"]
                .into_iter()
                .chain(self.cmd.split(char::is_whitespace).filter_map(|mut s| {
                    if s.is_empty() {
                        return None;
                    }
                    if s.starts_with('\"') {
                        s = &s[1..]
                    }
                    if s.ends_with('\"') {
                        s = &s[..(s.len() - 1)]
                    }
                    Some(s)
                }));
        duct::cmd(&*MATLA_BIN, args)
            .stderr_to_stdout()
            .stdout_capture()
            .unchecked()
    }

    /// Cleans the project directory.
    ///
    /// Used for post-testing cleanup.
    pub fn clean(&self) -> Result<(), String> {
        let proj_path = self.proj_path();
        let cmd = duct::cmd(&*MATLA_BIN, ["clean"])
            .dir(&proj_path)
            .stderr_to_stdout()
            .stdout_capture();
        let output = cmd
            .run()
            .with_context(|| anyhow!("running post-test cleanup on `{}`", proj_path.display()))
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            let mut err = format!(
                "Something went wrong while cleaning test project `{}`:\n",
                proj_path.display()
            );
            err.extend(String::from_utf8_lossy(&output.stdout).chars());
            Err(err)
        }
    }

    /// Runs the test.
    pub fn run(&self) -> TestRes {
        macro_rules! tryy {
            ($e:expr) => {
                match $e {
                    Ok(res) => res,
                    Err(e) => {
                        let mut res = TestRes::new(self);
                        res.add_error(e);
                        return res;
                    }
                }
            };
        }
        let cmd = self.cmd().dir(self.proj_path());
        let output = tryy! {
            cmd
                .run()
                .with_context(|| anyhow!("running `matla {}`", self.raw_cmd()))
                .with_context(|| anyhow!("on test {}", self.name_and_path()))
        };

        let mut res = TestRes::new(self);

        if let Some(code) = output.status.code() {
            res.check_code(code)
        } else {
            res.add_error(anyhow!("failed to retreive exit code"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        res.check_output(stdout);

        res
    }
}

/// Gathers the info required to present a test failure.
#[derive(Debug)]
pub struct TestRes<'a> {
    /// Test this result is for.
    pub test: &'a Test,
    /// The unexpected output, if any.
    pub unexpected_output: Option<String>,
    /// The unexpected exit code, if any.
    pub unexpected_code: Option<i32>,
    /// A list of other problems.
    pub misc: Vec<Error>,
}
impl<'a> std::ops::Deref for TestRes<'a> {
    type Target = Test;
    fn deref(&self) -> &Self::Target {
        &self.test
    }
}
impl<'a> TestRes<'a> {
    /// Constructor.
    pub fn new(test: &'a Test) -> Self {
        Self {
            test,
            unexpected_output: None,
            unexpected_code: None,
            misc: vec![],
        }
    }
    /// Checks whether the test's output is as expected.
    pub fn check_output(&mut self, output: Cow<str>) {
        let output = output.trim();
        if output != self.test.output() {
            debug_assert_eq!(self.unexpected_output, None);
            self.unexpected_output = Some(output.into())
        }
    }
    /// Checks whether the test's exit code is as expected.
    pub fn check_code(&mut self, code: i32) {
        if code != self.test.code() {
            debug_assert_eq!(self.unexpected_code, None);
            self.unexpected_code = Some(code)
        }
    }
    /// Adds a misc error.
    pub fn add_error(&mut self, err: impl Into<Error>) {
        self.misc.push(err.into())
    }

    /// True if the test was successful.
    pub fn is_success(&self) -> bool {
        self.unexpected_code.is_none() && self.unexpected_output.is_none() && self.misc.is_empty()
    }

    /// Pretty test failure report, `None` if not a failure.
    pub fn report(&self, style: &conf::Styles, pref: &str) -> Option<String> {
        if self.is_success() {
            return None;
        }

        let mut s = String::with_capacity(666);
        macro_rules! wrt {
            (nl $($tail:tt)*) => ({
                s.push('\n');
                wrt!($($tail)*);
            });
            (pref $($tail:tt)*) => ({
                s.push_str(pref);
                wrt!($($tail)*);
            });
            ($l:literal $(, $args:expr)+ $(,)?) => (
                s.push_str(&format!($l $(, $args)+))
            );
            ($l:literal) => (s.push_str($l));
        }
        let test = &self.test;

        wrt!(pref "| test file: `{}`",
            style.bold.paint(&test.path().display().to_string()),
        );
        wrt!(nl pref "| project directory: `{}`",
            style.bold.paint(&test.proj_path().display().to_string()),
        );
        wrt!(nl pref "> {} {}",
            style.bold.paint("matla"),
            test.cmd,
        );

        if let Some(code) = self.unexpected_code {
            wrt!(nl pref "+ got exit code {}, expected {}",
                style.fatal.paint(&code.to_string()),
                style.bold.paint(&test.code().to_string())
            );
        }

        if let Some(output) = self.unexpected_output.as_ref() {
            wrt!(nl pref "+ {} output",
                style.fatal.paint("unexpected"),
            );
            for ldiff in diff::lines(test.output(), output) {
                use diff::Result::*;
                let (style, pref, line) = match ldiff {
                    Left(line) => (&style.good, "-", line),
                    Right(line) => (&style.fatal, "+", line),
                    Both(line_1, line_2) => {
                        debug_assert_eq!(line_1, line_2);
                        (&style.comment, "=", line_1)
                    }
                };

                wrt!(nl pref "  {} | {}",
                    pref,
                    style.paint(line),
                );
            }
        }

        if !self.misc.is_empty() {
            let plural = if self.misc.len() > 1 { "s" } else { "" };
            wrt!(nl pref "+ {} {}{} occured",
                style.fatal.paint(&self.misc.len().to_string()),
                style.uline.paint("unexpected error"),
                style.uline.paint(plural),
            );
            for err in self.misc.iter() {
                for (idx, line) in format!("{:#?}", err).lines().enumerate() {
                    let line_pref = if idx == 0 { "" } else { "  " };
                    wrt!(nl pref "  {}{}",
                        line_pref,
                        line,
                    )
                }
            }
        }

        s.shrink_to_fit();
        Some(s)
    }
}
