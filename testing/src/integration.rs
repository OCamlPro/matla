//! Handles integration tests.

prelude!();

pub mod explain;
pub mod header;

use project::tlc::code;

/// Test outcome.
#[derive(Debug, Clone, Copy)]
pub enum ITestOutcome {
    Success,
    Violation(Violation),
    Failure(Failure),
    Error(ITestError),
}
impl ITestOutcome {
    pub fn to_exit_code(self) -> code::Exit {
        match self {
            Self::Success => code::Exit::Success,
            Self::Violation(v) => code::Exit::Violation(v.to_exit_code()),
            Self::Failure(f) => code::Exit::Failure(f.to_exit_code()),
            Self::Error(e) => code::Exit::Error(e.to_exit_code()),
        }
    }
}
/// Violation outcome.
#[derive(Debug, Clone, Copy)]
pub enum Violation {
    Assumption,
    Deadlock,
    Safety,
    Liveness,
    Assert,
}
impl Violation {
    pub fn to_exit_code(self) -> code::ExitViolation {
        match self {
            Self::Assumption => code::ExitViolation::ViolationAssumption,
            Self::Deadlock => code::ExitViolation::ViolationDeadlock,
            Self::Safety => code::ExitViolation::ViolationSafety,
            Self::Liveness => code::ExitViolation::ViolationLiveness,
            Self::Assert => code::ExitViolation::ViolationAssert,
        }
    }
}
/// Failure outcome.
#[derive(Debug, Clone, Copy)]
pub enum Failure {
    SpecEval,
    SafetyEval,
    LivenessEval,
}
impl Failure {
    pub fn to_exit_code(self) -> code::ExitFailure {
        match self {
            Self::SpecEval => code::ExitFailure::FailureSpecEval,
            Self::SafetyEval => code::ExitFailure::FailureSafetyEval,
            Self::LivenessEval => code::ExitFailure::FailureLivenessEval,
        }
    }
}
/// Error outcome.
#[derive(Debug, Clone, Copy)]
pub enum ITestError {
    SpecParse,
    ConfigParse,
    StatespaceTooLarge,
    System,
}
impl ITestError {
    pub fn to_exit_code(self) -> code::ExitError {
        match self {
            Self::SpecParse => code::ExitError::ErrorSpecParse,
            Self::ConfigParse => code::ExitError::ErrorConfigParse,
            Self::StatespaceTooLarge => code::ExitError::ErrorStatespaceTooLarge,
            Self::System => code::ExitError::ErrorSystem,
        }
    }
}

/// Integration test configuration.
#[derive(Debug, Clone)]
pub struct TestConf {
    /// If `Some(b)`, only compile in release if `b` and only in debug otherwise.
    pub only_in: Option<bool>,
    /// Expected outcome.
    pub expected: Option<ITestOutcome>,
}
impl Default for TestConf {
    fn default() -> Self {
        Self {
            only_in: None,
            expected: None,
        }
    }
}
impl TestConf {
    /// Debug-only accessor.
    pub fn debug_only(&self) -> bool {
        self.only_in.map(|b| !b).unwrap_or(false)
    }
    /// Release-only accessor.
    pub fn release_only(&self) -> bool {
        self.only_in.unwrap_or(false)
    }
    /// Expected outcome accessor.
    pub fn expected(&self) -> ITestOutcome {
        self.expected.clone().unwrap_or(ITestOutcome::Success)
    }

    /// True if active given the release flag.
    pub fn is_active(&self, release: bool) -> bool {
        match (self.debug_only(), self.release_only(), release) {
            (true, _, true) | (_, true, false) => false,
            _ => true,
        }
    }
}

/// Test library configuration.
#[derive(Debug, Clone)]
pub struct LibConf;

/// An integration test.
#[derive(Debug, Clone)]
pub struct Test {
    /// Module name.
    pub module_name: String,
    /// Module path under the `tests` directory.
    pub module_path_pref: io::PathBuf,
    /// Module path: `<module_path_pref>/<module_name>.tla`.
    pub module_path: String,
    /// Path to the TLA file.
    pub tla_path: io::PathBuf,
    /// Path to the cfg file.
    pub cfg_path: io::PathBuf,
    /// Test configuration.
    pub conf: TestConf,
}
impl Test {
    /// Loads a test.
    pub fn new(
        from: impl AsRef<io::Path>,
        tla_path: impl Into<io::PathBuf>,
        conf: TestConf,
    ) -> Res<Self> {
        let tla_path = tla_path.into();
        if !tla_path.is_file() {
            bail!("`{}` does not exist or is not a file", tla_path.display());
        }
        let from = from.as_ref();
        let cfg_path = {
            let mut path = tla_path.clone();
            path.set_extension("cfg");
            path
        };
        if !cfg_path.is_file() {
            bail!(
                "`cfg` file `{}` for test `{}` does not exist or is not a file",
                cfg_path.display(),
                tla_path.display(),
            );
        }

        let (module_name, module_path_pref) = {
            let stem = io::file_stem(&tla_path)?;
            let mut path: io::PathBuf = tla_path
                .strip_prefix(from)
                .with_context(|| {
                    anyhow!(
                        "failed to construct module path for test `{}` from `{}`",
                        tla_path.display(),
                        from.display(),
                    )
                })?
                .into();
            let okay = path.pop();
            if !okay {
                bail!(
                    "failed to construct module path for test `{}` from `{}`",
                    tla_path.display(),
                    from.display(),
                );
            }

            (stem, path)
        };

        let module_path = format!("{}/{}.tla", module_path_pref.display(), module_name);

        Ok(Self {
            module_path,
            module_name,
            module_path_pref,
            tla_path,
            cfg_path,
            conf,
        })
    }

    /// True if the test is active given the release flag and filter optional set.
    pub fn is_active(&self, release: bool, filter: Option<&Filter>) -> bool {
        if !self.conf.is_active(release) {
            false
        } else {
            filter
                .map(|filter| filter.contains(&self.module_path))
                .unwrap_or(true)
        }
    }

    /// Module path: path prefix `/` module name.
    pub fn module_path(&self) -> &str {
        &self.module_path
    }

    /// Module path, but with all `/` replaced by `__`.
    pub fn module_path_as_ident(&self) -> String {
        let path = format!("{}/{}", self.module_path_pref.display(), self.module_name);
        path.replace('/', "__")
    }

    /// Runs the test.
    pub fn run(
        &self,
        mut proj: project::SourceProject,
        release: bool,
        libs: &[TestLib],
    ) -> Res<Result<(), Vec<String>>> {
        // Add tla and cfg files for this tests.
        let tla_idx = proj.add_file(&self.tla_path)?;
        let _cfg_idx = proj.add_file(&self.cfg_path)?;
        // Add all test libraries.
        for lib in libs.iter() {
            let _ = proj
                .add_file(&lib.tla_path)
                .context("failed to add test library")?;
        }

        let test_dir = format!("integration_test_{}", self.module_path_as_ident());
        let target = conf::Target::new_test(proj.path()?, release, &test_dir);
        let entry = proj[tla_idx].module().to_string();
        let tlc_cla = conf::TlcCla::default()
            .seed(0)
            .workers(Some(1))
            .terse(true)
            .diff_cexs(true)
            .into_customl(conf::customl::Source::Custom("internal test configuration"));
        let (project, _) = proj.into_full(Some(entry), target, Some(&tlc_cla))?;

        let mut tlc_out = TlcOutputHandler::new();
        let tlc = project.run_tlc_async(&mut tlc_out)?;

        let outcome = tlc.run()?;
        let project::tlc::ProcessOutcome { code, status } = outcome.process;
        let expected = self.conf.expected().to_exit_code().code();

        if status.as_ref().map(|c| c.code()) == Some(expected) {
            return Ok(Ok(()));
        }

        let mut error = vec![format!(
            "test `{}` ({})",
            self.module_name,
            project.source[tla_idx].path().display()
        )];
        if !libs.is_empty() {
            error.push("with test library/ies".into());
            for lib in libs.iter() {
                error.push(format!("- `{}`", lib.tla_path.display()));
            }
        }
        error.push(format!("expected exit code to be `{}`", expected));
        error.push(if let Some(exit) = status {
            format!("but got `{}`", exit)
        } else {
            format!("but got unknown exit code {}", code)
        });
        error.push("".into());
        error.push("|===| TLC output:".into());
        error.extend(tlc_out.lines().into_iter().map(|s| format!("| {}", s)));
        error.push("|===|".into());

        Ok(Err(error))
    }
}

/// A library for tests to use.
pub struct TestLib {
    /// Configuration.
    pub conf: LibConf,
    /// Module name.
    pub module_name: String,
    /// Module path under the `tests` directory.
    pub module_path_pref: io::PathBuf,
    /// Module path: `<module_path_pref>/<module_name>`.
    pub module_path: String,
    /// Path to the TLA file.
    pub tla_path: io::PathBuf,
}
impl TestLib {
    /// Constructor.
    pub fn new(
        path: impl Into<io::PathBuf>,
        from: impl AsRef<io::Path>,
        conf: LibConf,
    ) -> Res<Self> {
        let path = path.into();
        let from = from.as_ref();
        let (module_name, module_path_pref) = {
            let stem = io::file_stem(&path)?;
            let mut path: io::PathBuf = path
                .strip_prefix(from)
                .with_context(|| {
                    anyhow!(
                        "failed to construct module path for test `{}` from `{}`",
                        path.display(),
                        from.display(),
                    )
                })?
                .into();
            let okay = path.pop();
            if !okay {
                bail!(
                    "failed to construct module path for test `{}` from `{}`",
                    path.display(),
                    from.display(),
                );
            }

            (stem, path)
        };
        let module_path = {
            let pref = module_path_pref.display().to_string();
            if pref.is_empty() {
                module_name.clone()
            } else {
                format!("{}/{}", pref, module_name)
            }
        };

        // Check there's no `cfg` file associated.
        {
            let mut cfg_path = path.clone();
            cfg_path.set_extension("cfg");
            if cfg_path.exists() {
                log::warn!(
                    "test library `{}` has an associated `cfg` file `{}`",
                    path.display(),
                    cfg_path.display(),
                );
                log::warn!("please delete it, it does not make sens for a test library");
            }
        }

        Ok(Self {
            module_name,
            module_path_pref,
            module_path,
            tla_path: path,
            conf,
        })
    }
}

/// Integration test context.
pub struct Cxt<'a> {
    /// Actual tests.
    pub tests: Map<io::PathBuf, (Vec<Test>, Vec<TestLib>)>,
    /// Pending `cfg` files.
    pub pending_cfg: Set<io::PathBuf>,
    /// Optional set of modules to keep.
    pub filter: Option<&'a Filter>,
    /// Source project.
    pub src_project: &'a project::SourceProject,
    /// Release mode flag.
    pub release: bool,
}
impl<'a> Cxt<'a> {
    /// Constructor.
    pub(crate) fn new(
        src_project: &'a project::SourceProject,
        release: bool,
        filter: Option<&'a Filter>,
    ) -> Self {
        Self {
            tests: Map::new(),
            pending_cfg: Set::new(),
            filter,
            src_project,
            release,
        }
    }

    /// Number of active tests in the context.
    pub fn test_count(&self) -> usize {
        self.tests.values().map(|(tests, _libs)| tests.len()).sum()
    }

    /// Iterates over all files in a directory, recursively if `recursive` is `true`.
    pub fn dir_files_do(
        dir: impl Into<io::PathBuf>,
        recursive: bool,
        mut action: impl FnMut(&io::Path) -> Res<()>,
    ) -> Res<()> {
        let dir = dir.into();
        if !recursive {
            for entry in dir.read_dir().with_context(|| {
                anyhow!("failed to read entries in directory `{}`", dir.display())
            })? {
                let entry =
                    entry.with_context(|| anyhow!("reading an entry in `{}`", dir.display()))?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    continue;
                }

                action(&entry_path)?
            }
        } else {
            for entry in WalkDir::new(&dir) {
                let entry =
                    entry.with_context(|| anyhow!("reading an entry in `{}`", dir.display()))?;
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    continue;
                }

                action(&entry_path)?
            }
        }
        Ok(())
    }

    /// Loads all tests in a directory.
    ///
    /// - `filter` is an optional set of module names causing to ignore all modules not in the set;
    /// - `release` is necessary to ignore `debug_only`/`release_only` tests;
    /// - `recursive`, if `true`, loads tests in sub-directories recursively.
    pub fn dir_load(
        root_dir: impl Into<io::PathBuf>,
        src_project: &'a project::SourceProject,
        filter: Option<&'a Filter>,
        release: bool,
        recursive: bool,
    ) -> Res<Self> {
        let root_dir = root_dir.into();
        if !root_dir.is_dir() {
            bail!(
                "`{}` does not exist or is not a directory",
                root_dir.display()
            )
        }

        let mut cxt = Self::new(src_project, release, filter);

        Self::dir_files_do(&root_dir, recursive, |entry_path| {
            // Only keep going if entry is a TLA file.
            match io::file_ext(&entry_path) {
                Some(ext) if ext == "tla" => {
                    // Keep going.
                    ()
                }
                Some(ext) if ext == "cfg" => {
                    // Register and skip.
                    let _is_new = cxt.pending_cfg.insert(entry_path.into());
                    if !_is_new {
                        bail!(
                            "trying to register `cfg` file `{}` twice",
                            entry_path.display()
                        )
                    }
                    return Ok(());
                }
                Some(_) | None => {
                    // What is this doing here?
                    log::warn!(
                        "found unexpected file `{}` in integration test directory `{}`",
                        entry_path.display(),
                        root_dir.display(),
                    );
                    return Ok(());
                }
            }

            let content = io::load_file(&entry_path)?;
            let workdir = {
                let mut path = io::PathBuf::from(entry_path);
                let success = path.pop();
                if !success {
                    bail!(
                        "failed to retrieve parent directory of path `{}`",
                        path.display()
                    );
                }
                path
            };

            match header::parse(&content) {
                // Loading an actual test.
                Ok(Left(conf)) => {
                    let test = Test::new(&root_dir, entry_path, conf).with_context(|| {
                        anyhow!("failed to load test `{}`", entry_path.display())
                    })?;
                    cxt.tests
                        .entry(workdir.clone())
                        .or_insert_with(|| (vec![], vec![]))
                        .0
                        .push(test);
                }
                // Loading a test library.
                Ok(Right(conf)) => {
                    let lib = TestLib::new(entry_path, &root_dir, conf)?;
                    cxt.tests
                        .entry(workdir.clone())
                        .or_insert_with(|| (vec![], vec![]))
                        .1
                        .push(lib);
                }
                Err(peg_error) => {
                    let (expected, pos) = (
                        peg_error.expected,
                        source::Pos::from_peg(peg_error.location),
                    );
                    let content = io::load_file(&entry_path)?;
                    let pretty = pos
                        .pretty(&content, Some(&format!("expected {}", expected)))?
                        .into_iter()
                        .fold(String::new(), |acc, next| {
                            if acc.is_empty() {
                                next
                            } else {
                                format!("{}\n{}", acc, next)
                            }
                        });
                    bail!(base::Error::msg(explain::test_conf())
                        .context(pretty)
                        .context(anyhow!(
                            "illegal test configuration in file `{}` at {}",
                            entry_path.display(),
                            pos
                        )))
                }
            }

            Ok(())
        })?;

        cxt.finalize()?;

        Ok(cxt)
    }

    /// Finalizes the context.
    ///
    /// - removes inactive tests.
    ///
    /// Errors:
    ///
    /// - clash between test libraries and modules in the source project.
    ///
    /// Warnings:
    ///
    /// - pending `cfg` files with no associated test module.
    pub fn finalize(&mut self) -> Res<()> {
        use rayon::prelude::*;
        // Check pending `cfg` files.
        for cfg_file in mem::replace(&mut self.pending_cfg, Set::new()).into_iter() {
            let okay = self
                .tests
                .values()
                .par_bridge()
                .any(|(tests, _libs)| tests.par_iter().any(|test| test.cfg_path == cfg_file));
            if !okay {
                log::warn!(
                    "`cfg` file `{}` is not associated to any test",
                    cfg_file.display(),
                )
            }
        }

        // Check for clashes with source modules.
        let errors = sync::RwLock::new(0);
        let err_inc = || {
            *errors.write().expect("error RwLock is poisoned") += 1;
        };
        self.tests.values().par_bridge().for_each(|(tests, libs)| {
            // Check for module-clashes between test files and source files.
            tests.par_iter().for_each(|test| {
                if let Some(idx) = self.src_project.top_modules.get(&test.module_name).cloned() {
                    log::error!(
                        "test file `{}` defines module `{}` which clashes with source file `{}`",
                        test.tla_path.display(),
                        test.module_name,
                        self.src_project[idx].path.display(),
                    );
                    err_inc()
                }
            });
            // Check for module-clashes between test libs and source files.
            libs.par_iter().for_each(|lib| {
                if let Some(idx) = self.src_project.top_modules.get(&lib.module_name).cloned() {
                    log::error!(
                        "test module `{}` defines module `{}` which clashes with source file `{}`",
                        lib.tla_path.display(),
                        lib.module_name,
                        self.src_project[idx].path.display(),
                    );
                    err_inc()
                }
            })
        });

        let errors = errors.into_inner().expect("error RwLock is poisoned");
        if errors > 0 {
            bail!("integration test loading failed with {} error(s)", errors)
        }

        // Removes tests ignored by `filter` or by `release`.
        self.tests.retain(|_key, (tests, _libs)| {
            for test_idx in (0..tests.len()).rev() {
                if !tests[test_idx].is_active(self.release, self.filter) {
                    let _ = tests.remove(test_idx);
                }
            }
            !tests.is_empty()
        });

        Ok(())
    }

    /// Runs the tests.
    pub fn run<'me, T, Action>(&'me self, parallel: bool, action: Action) -> Res<Vec<T>>
    where
        T: Send + 'me,
        Action: Fn(Res<TestRes>, &'me Test) -> T + Sync,
    {
        if !self.pending_cfg.is_empty() {
            bail!("trying to run integration tests before context finalization");
        }

        let res: Vec<T> = if parallel {
            use rayon::prelude::*;

            self.tests
                .values()
                .par_bridge()
                .map(|(tests, libs)| {
                    tests.par_iter().map(|test| {
                        let res = test.run(self.src_project.clone(), self.release, libs);
                        action(res, test)
                    })
                })
                .flatten()
                .collect()
        } else {
            self.tests
                .values()
                .map(|(tests, libs)| {
                    tests.iter().map(|test| {
                        let res = test.run(self.src_project.clone(), self.release, libs);
                        action(res, test)
                    })
                })
                .flatten()
                .collect()
        };

        Ok(res)
    }
}

/// Aggregates everything said by TLC.
pub struct TlcOutputHandler {
    pub lines: Vec<String>,
    pub cexs: Vec<cex::Cex>,
    pub errors: Vec<project::tlc::TlcError>,
    pub outcome: Option<tlc::RunOutcome>,
}
impl TlcOutputHandler {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            lines: Vec::with_capacity(113),
            cexs: vec![],
            errors: vec![],
            outcome: None,
        }
    }
    pub fn lines(self) -> Vec<String> {
        self.lines
    }
}
impl project::tlc::Out for TlcOutputHandler {
    fn handle_message(&mut self, msg: &project::tlc::msg::Msg, _log_level: log::Level) {
        self.lines.extend(msg.lines().into_iter().map(String::from))
    }
    fn handle_outcome(&mut self, outcome: tlc::RunOutcome) {
        self.outcome = Some(outcome)
    }
    fn handle_error(&mut self, error: impl Into<tlc::TlcError>) -> Res<()> {
        self.errors.push(error.into());
        Ok(())
    }
    fn handle_cex(&mut self, cex: cex::Cex) {
        self.cexs.push(cex);
    }
}
