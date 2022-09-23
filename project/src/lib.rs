//! Project management.

#![forbid(rustdoc::broken_intra_doc_links)]

/// This crate's prelude.
pub mod prelude {
    pub use base::*;
    pub use cex;
    pub use conf;

    pub use crate::{
        idx,
        tlc::{self, outcome::*, TlcRun},
        FullProject, ModuleOrTop, Project, SourceProject, TargetProject,
    };
}
/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    {$($stuff:tt)*} => {
        use $crate::prelude::{*, $($stuff)*};
    };
}

pub mod matla;
pub mod tlc;

prelude!();

/// Explanations about certain notions, used in error-reporting.
pub mod explain {
    /// Explains what *runnable module* means.
    pub fn runnable_module() -> &'static str {
        "a module `module` is *runnable* if both `module.tla` and `module.cfg` exist"
    }
}

/// Type-safe indices.
pub mod idx {
    prelude!();

    safe_index::new! {
        /// Index for a file in the project directory.
        File,
        /// Map from file indices to something.
        map: Files,
        /// A set of files.
        btree set: FileSet,
        /// A btree map from files.
        btree map: FileBMap,
    }
}

/// Either stores a module or unit-variants representing either the top TLA or the top CFG file.
#[derive(Debug, Clone)]
pub enum ModuleOrTop {
    Module(String),
    TopTla,
    TopCfg,
}
implem! {
    for ModuleOrTop {
        From<String> { |t| Self::new(t) }
        Display { |&self, fmt| match self {
            Self::Module(module) => module.fmt(fmt),
            Self::TopTla => "<top tla>".fmt(fmt),
            Self::TopCfg => "<top cfg>".fmt(fmt),
        }}
    }
}
impl ModuleOrTop {
    /// Module constructor.
    pub fn new(t: impl Into<String>) -> Self {
        Self::Module(t.into())
    }
    /// Top TLA file.
    pub fn top_tla() -> Self {
        Self::TopTla
    }
    /// Top CFG file.
    pub fn top_cfg() -> Self {
        Self::TopCfg
    }

    /// Yields the source file.
    pub fn to_source<'me, 'project>(
        &'me self,
        proj: &'project FullProject,
    ) -> Res<&'project TlaFile> {
        match self {
            Self::Module(module) => proj
                .source_tla_file(module)
                .ok_or_else(|| anyhow!("unknown module `{}`", module)),
            Self::TopTla => proj.source_tla_file(&proj.actual_entry).ok_or_else(|| {
                anyhow!(
                    "project does not know its own entry `{}`",
                    proj.actual_entry
                )
            }),
            Self::TopCfg => {
                let entry = &proj.actual_entry;
                let tla_idx = proj
                    .source
                    .top_modules
                    .get(entry)
                    .ok_or_else(|| anyhow!("unknown entry module `{}`", entry))?;
                let cfg_idx = proj
                    .source
                    .tla_to_cfg
                    .get(tla_idx)
                    .cloned()
                    .ok_or_else(|| anyhow!("entry point `{}` has no cfg file", entry))?;
                Ok(&proj.source[cfg_idx])
            }
        }
    }

    /// Yields the target file.
    pub fn to_target<'me, 'project>(
        &'me self,
        proj: &'project FullProject,
    ) -> Res<&'project TlaFile> {
        match self {
            Self::Module(module) => proj
                .target_tla_file(module)
                .ok_or_else(|| anyhow!("unknown module `{}`", module)),
            Self::TopTla => {
                let entry = &proj.actual_entry;
                proj.target_tla_file(entry)
                    .ok_or_else(|| anyhow!("project does not know its own entry point `{}`", entry))
            }
            Self::TopCfg => {
                let entry = &proj.actual_entry;
                let tla_idx = proj.target.top_modules.get(entry).ok_or_else(|| {
                    anyhow!("project does not know its own entry point `{}`", entry)
                })?;
                let cfg_idx = proj
                    .target
                    .tla_to_cfg
                    .get(tla_idx)
                    .cloned()
                    .ok_or_else(|| anyhow!("entry point `{}` has no cfg file", entry))?;
                Ok(&proj.target[cfg_idx])
            }
        }
    }
}

/// A TLA or TLA config file.
#[readonly]
#[derive(Debug, Clone)]
pub struct TlaFile {
    /// This file's index.
    pub idx: idx::File,
    /// File path.
    pub path: io::PathBuf,
    /// Name of the module corresponding to this file.
    pub module: String,
    /// True if the file is a TLA (not TLA config) file.
    pub is_tla: bool,
}
implem! {
    for TlaFile {
        Display {
            |&self, fmt| write!(
                fmt,
                "[{}] {} ({}, {})",
                self.idx,
                self.path.display(),
                self.module,
                if self.is_tla { "tla" } else { "cfg" },
            )
        }
    }
}
impl TlaFile {
    /// Extension of TLA files.
    pub const TLA_FILE_EXT: &'static str = "tla";
    /// Extension of TLA config files.
    pub const CFG_FILE_EXT: &'static str = "cfg";

    /// Constructor from a file path.
    ///
    /// - Returns `None` if the file does not have a `tla` or `cfg` extension.
    /// - Does **not** fail if the file does not exist.
    pub fn new(idx: idx::File, path: impl Into<io::PathBuf>) -> Res<Option<Self>> {
        let path: io::PathBuf = path.into();
        let ext = if let Some(ext) = io::file_ext(&path) {
            ext
        } else {
            return Ok(None);
        };
        let is_tla = if ext == Self::TLA_FILE_EXT {
            true
        } else if ext == Self::CFG_FILE_EXT {
            false
        } else {
            return Ok(None);
        };
        let module = io::file_stem(&path)?;
        let res = Self {
            path,
            is_tla,
            module,
            idx,
        };
        res.check()?;

        Ok(Some(res))
    }

    /// File name.
    pub fn file_name(&self) -> Res<&std::ffi::OsStr> {
        self.path
            .file_name()
            .ok_or_else(|| anyhow!("failed to retrieve file name of `{}`", self.path.display()))
    }

    /// Path to this file.
    pub fn path(&self) -> &io::PathBuf {
        &self.path
    }
    /// True if the file is a TLA (not TLA config) file.
    pub fn is_tla(&self) -> bool {
        self.is_tla
    }
    /// True if the file is a TLA config (not TLA) file.
    pub fn is_cfg(&self) -> bool {
        !self.is_tla
    }
    /// Module associated to this file.
    pub fn module(&self) -> &str {
        &self.module
    }

    /// True if this file corresponds to the `Matla` module.
    pub fn is_matla_module(&self) -> bool {
        self.module == "Matla"
    }

    /// Changes its non-file-name path.
    fn change_path(&self, target: impl Into<io::PathBuf>) -> Res<Self> {
        let mut path = target.into();
        path.push(io::file_name(&self.path)?);
        let res = Self {
            idx: self.idx,
            path,
            module: self.module.clone(),
            is_tla: self.is_tla,
        };
        res.check()?;
        Ok(res)
    }

    /// Fails on broken invariants.
    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn check(&self) -> Res<()> {
        Ok(())
    }
    /// Fails on broken invariants.
    #[cfg(debug_assertions)]
    pub fn check(&self) -> Res<()> {
        let ext = if let Some(ext) = io::file_ext(&self.path) {
            ext
        } else {
            bail!(
                "failed to retrieve extension of file `{}`",
                self.path.display()
            )
        };
        if ext == Self::TLA_FILE_EXT {
            if !self.is_tla {
                bail!(
                    "`{}` has a `{}` extension but is not considered a TLA file",
                    self.path.display(),
                    Self::TLA_FILE_EXT,
                )
            }
        } else if ext == Self::CFG_FILE_EXT {
            if self.is_tla {
                bail!(
                    "`{}` has a `{}` extension but is not considered a TLA config file",
                    self.path.display(),
                    Self::CFG_FILE_EXT,
                )
            }
        } else {
            bail!(
                "`{}` is not a TLA (config) file: expected extension to be `{}` or `{}`",
                self.path.display(),
                Self::TLA_FILE_EXT,
                Self::CFG_FILE_EXT,
            )
        }
        Ok(())
    }

    /// Runs TLC on a file.
    fn run_tlc(&self, mut tlc_cmd: io::Command) -> Res<io::Output> {
        self.check()?;
        if !self.is_tla() {
            bail!("trying to run TLC on a cfg file `{}`", self.path.display())
        }

        tlc_cmd
            .arg(&self.file_name()?)
            .output()
            .with_context(|| format!("running TLC on `{}`", self.path.display()))
    }

    /// Runs TLC on a file, async mode.
    fn run_tlc_async<Out: tlc::Out>(
        &self,
        mut tlc_cmd: io::Command,
        handler: Out,
    ) -> Res<TlcRun<Out>> {
        self.check()?;
        if !self.is_tla() {
            bail!("trying to run TLC on a cfg file `{}`", self.path.display())
        }

        tlc_cmd.arg(&self.file_name()?);

        Ok(TlcRun::new(tlc_cmd, handler))
    }

    /// Completes a TLC command.
    fn complete_tlc_cmd(&self, mut tlc_cmd: io::Command) -> Res<io::Command> {
        self.check()?;
        if !self.is_tla() {
            bail!("trying to run TLC on a cfg file `{}`", self.path.display())
        }
        tlc_cmd.arg(self.module());
        Ok(tlc_cmd)
    }
}

/// Unit type representing a *source* project, *i.e.* actual files from the user.
///
/// Used as type parameter for [`Project`] to distinguish from [`Target`] projects.
#[derive(Debug, Clone, Copy)]
pub struct Source;
/// Unit type representing a *target* project, *i.e.* where we run/build/test/doc.
///
/// Used as type parameter for [`Project`] to distinguish from [`Source`] projects.
#[derive(Debug, Clone, Copy)]
pub struct Target;

/// A source project.
pub type SourceProject = Project<Source>;
/// A target project.
pub type TargetProject = Project<Target>;

/// A full TLA project.
#[readonly]
#[derive(Debug, Clone)]
pub struct Project<Kind> {
    /// Project path.
    pub path: Option<io::PathBuf>,
    /// All files appearing in the project
    pub files: idx::Files<TlaFile>,
    /// Module view of the project's TLA (not TLA cfg) files.
    pub top_modules: Map<String, idx::File>,
    /// Maps TLA (not TLA cfg) files that have a corresponding TLA cfg file to that cfg file.
    pub tla_to_cfg: idx::FileBMap<idx::File>,
    /// CFG files waiting for a TLA file.
    pub pending_cfg: Map<String, idx::File>,
    /// Phantom data for the project kind.
    _kind_phantom: PhantomData<Kind>,
}

impl<K> std::ops::Index<idx::File> for Project<K> {
    type Output = TlaFile;
    fn index(&self, idx: idx::File) -> &Self::Output {
        &self.files[idx]
    }
}

impl<K> Project<K> {
    /// Path to the project directory.
    pub fn path(&self) -> Res<io::PathBuf> {
        self.path
            .clone()
            .map(Ok)
            .unwrap_or_else(conf::top_cla::project_path)
    }

    /// Fails if `module` is an unknown module and has a `cfg` file.
    pub fn check_module_runnable(&self, module: impl AsRef<str>) -> Res<()> {
        let module = module.as_ref();
        let idx = self
            .top_modules
            .get(module)
            .ok_or_else(|| anyhow!("module `{}` does not exist in this project", module))?;
        if !self.tla_to_cfg.contains_key(idx) {
            bail!(anyhow!(explain::runnable_module())
                .context(anyhow!("module `{}` is not runnable", module)))
        }
        Ok(())
    }

    /// Adds a new file to the project.
    pub fn add_file(&mut self, path: impl Into<io::PathBuf>) -> Res<idx::File> {
        let path = path.into();
        self.try_add_file(&path)?
            .ok_or_else(|| anyhow!("`{}` is not a legal TLA or TLA cfg file", path.display()))
    }

    /// Adds a new file to the project.
    pub fn try_add_file(&mut self, path: impl Into<io::PathBuf>) -> Res<Option<idx::File>> {
        let path = path.into();
        let next_idx = self.files.next_index();
        if let Some(tla_file) = TlaFile::new(next_idx, &path)? {
            log::trace!("adding tla/cfg file `{}`", path.display());
            let idx = self.files.push(tla_file);
            debug_assert_eq!(next_idx, idx);

            let file = &self.files[idx];

            if file.is_tla() {
                log::trace!("registering top-module `{}` for `{}`", file.module(), file);
                let prev = self.top_modules.insert(file.module().to_string(), idx);
                if let Some(prev_idx) = prev {
                    bail!(
                        "files `{}` and `{}` both define the same module",
                        self.files[prev_idx],
                        file,
                    )
                }

                // Check if a cfg file is waiting for this TLA file.
                if let Some(cfg) = self.pending_cfg.remove(file.module()) {
                    self.tla_to_cfg.insert(file.idx, cfg);
                }
            } else {
                // Do we have a TLA file corresponding to this cfg's module?
                if let Some(tla_idx) = self.top_modules.get(file.module()).cloned() {
                    log::trace!(
                        "registering cfg file `{}` as associated with `{}`",
                        file,
                        self.files[tla_idx],
                    );
                    let prev = self.tla_to_cfg.insert(tla_idx, file.idx);
                    if let Some(prev_idx) = prev {
                        bail!(
                            "TLA file `{}` seems to have two cfg files: `{}` and `{}`",
                            self.files[tla_idx],
                            self.files[prev_idx],
                            file,
                        )
                    }
                } else {
                    // No TLA file for this cfg file, insert in pending.
                    let prev = self.pending_cfg.insert(file.module().to_string(), file.idx);
                    if let Some(prev_idx) = prev {
                        bail!(
                            "cfg files `{}` and `{}` seem to define the same module",
                            self.files[prev_idx],
                            file,
                        )
                    }
                }
            }
            Ok(Some(next_idx))
        } else {
            Ok(None)
        }
    }

    /// If exactly one module is runnable (has a cfg), returns the name of that module.
    pub fn get_unique_runnable_module(&self) -> Option<&str> {
        if self.tla_to_cfg.len() != 1 {
            return None;
        }
        let (idx, _) = self
            .tla_to_cfg
            .iter()
            .next()
            .expect("single next of iterator over exactly one element cannot fail");
        Some(&self.files[*idx].module())
    }
    /// True if at least one module is runnable.
    pub fn has_runnable_modules(&self) -> bool {
        !self.tla_to_cfg.is_empty()
    }
    /// Validates a runnable module, or extracts the only runnable module.
    ///
    /// Fails if
    /// - `main = Some(main)` and `main` is not a runnable module,
    /// - `main = None` and there is not **exactly** one runnable module in the project.
    pub fn validate_runnable_module<'a>(&'a self, main: Option<&'a str>) -> Res<&'a str> {
        if let Some(main) = main {
            log::debug!("checking user-provided main module `{}`", main);
            self.check_module_runnable(&main)?;
            Ok(main)
        } else if let Some(main) = self.get_unique_runnable_module() {
            Ok(main)
        } else if self.has_runnable_modules() {
            let command_example = {
                let args = std::env::args();
                let mut s = String::new();
                for arg in args {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str(&arg)
                }
                let first_runnable_module = self
                    .tla_to_cfg
                    .keys()
                    .cloned()
                    .next()
                    .map(|idx| self[idx].module())
                    .expect("project has runnable projects");
                s.push(' ');
                s.push_str(first_runnable_module);
                s
            };
            let mut e = anyhow!(
                "please specify which module to run with, for example `{}`",
                command_example,
            );
            e = e.context(explain::runnable_module());
            for idx in self.tla_to_cfg.keys().cloned() {
                e = e.context(anyhow!("- `{}`", self[idx].module()));
            }
            bail!(e.context(anyhow!("there are more than one runnable module:")));
        } else {
            bail!(Error::msg(explain::runnable_module())
                .context("this project has no runnable module, aborting"))
        }
    }

    /// Check that all files that are supposed to exist actually do.
    pub fn check_files_exist(&self) -> Res<()> {
        let mut not_there = None;
        for file in self.files.iter() {
            if !file.path().is_file() {
                not_there.get_or_insert_with(Vec::new).push(file.clone());
            }
        }
        if let Some(not_there) = not_there {
            let mut err = anyhow!("{} project file(s) not found", not_there.len());
            for file in not_there {
                err = err.context(format!(
                    "file `{}` not found or is a directory",
                    file.path().display()
                ));
            }
            let path = self
                .path()
                .context("while retrieving project path")
                .context("while handling errors")?;
            return Err(err.context(format!("error in project at `{}`", path.display())));
        }
        Ok(())
    }

    /// Changes the whole path of a project.
    fn change_path(&self, target: impl Into<io::PathBuf>) -> Res<TargetProject> {
        let path = target.into();
        let mut files = idx::Files::with_capacity(self.files.len());

        for file in self.files.iter() {
            let file = file.change_path(&path)?;
            let file_idx = file.idx;
            let idx = files.push(file);
            debug_assert_eq!(file_idx, idx);
        }

        let res = TargetProject {
            path: Some(path),
            files,
            top_modules: self.top_modules.clone(),
            tla_to_cfg: self.tla_to_cfg.clone(),
            pending_cfg: self.pending_cfg.clone(),
            _kind_phantom: PhantomData,
        };
        res.check_files_exist()?;

        Ok(res)
    }

    /// Matla module file, if any.
    pub fn matla_module_file(&self) -> Option<&TlaFile> {
        if let Some(idx) = self.top_modules.get(matla::MATLA_MODULE_NAME).cloned() {
            Some(&self[idx])
        } else {
            None
        }
    }

    fn inner_from_path(path: impl Into<io::PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
            files: idx::Files::with_capacity(17),
            top_modules: Map::new(),
            tla_to_cfg: idx::FileBMap::new(),
            pending_cfg: Map::new(),
            _kind_phantom: PhantomData,
        }
    }
}

impl SourceProject {
    /// Constructor from a project path.
    pub fn from_path(path: impl AsRef<io::Path>) -> Res<Self> {
        let path = io::PathBuf::from(path.as_ref());

        let mut slf = Self::inner_from_path(&path);

        'tla_files: for entry in path.read_dir().with_context(|| {
            format!(
                "failed to read directory entries for project path `{}`",
                path.display()
            )
        })? {
            let entry = entry.with_context(|| {
                format!(
                    "failed to read an entry of project path `{}`",
                    path.display()
                )
            })?;
            let entry_path = entry.path();
            log::debug!("entry: `{}`", entry_path.display());

            if !entry_path.is_file() {
                log::trace!("skipping, entry's not a file");
                continue 'tla_files;
            }

            slf.try_add_file(entry_path)?;
        }

        // Scan pending cfg files and issue warnings if non-empty.
        if !slf.pending_cfg.is_empty() {
            log::warn!(
                "{} cfg file(s) have no associated TLA file:",
                slf.pending_cfg.len()
            );
            for (_, idx) in slf.pending_cfg.iter() {
                log::warn!("- {}", slf.files[*idx].path.display());
            }
        } else {
            log::debug!("no pending cfg file left");
        }

        Ok(slf)
    }

    /// Path to the toml config file of the project (may not exist).
    pub fn toml_config_path(&self) -> Option<io::PathBuf> {
        self.path.clone().map(|mut path| {
            path.push(conf::project::TOML_CONFIG_FILENAME);
            path
        })
    }

    /// Loads the toml config file of the project, fails with an explanation if none.
    pub fn load_toml_config(&self) -> Res<conf::Project> {
        if let Some(path) = self.toml_config_path() {
            if !path.exists() {
                bail!(
                    Error::msg("make sure you run `matla init` before `matla run ...`")
                        .context("cannot run matla without a project configuration file")
                        .context(format!(
                            "could not find project configuration file `{}`",
                            path.display(),
                        ))
                )
            }
            if path.is_dir() {
                bail!(
                    Error::msg("please delete/move this directory and run `matla init`").context(
                        format!(
                        "`{}` should be the project's configuration file, but it is a directory",
                        path.display(),
                    )
                    )
                )
            }

            conf::project::raw_load(path)
        } else {
            conf::project::try_read(|conf| conf.clone())
        }
    }

    /// Builds a full project.
    ///
    /// This involves creating a second, target project where the build/run/doc/test-ing will take
    /// place. Everything is synchronized, *i.e.* files from the `self` (user) project have been
    /// synchronized with the ones in the target project.
    ///
    /// See [`Self::to_target`] for more details.
    pub fn into_full(
        self,
        entry: Option<String>,
        target_conf: conf::Target,
        tlc_cla: Option<&conf::customl::TlcCla>,
    ) -> Res<(FullProject, conf::customl::TlcCla)> {
        let target = &target_conf.build_path;
        let release = target_conf.release;
        let target = self.to_target(target, release)?;
        let conf = self.load_toml_config()?;
        FullProject::new(entry, target_conf, self, conf, target, tlc_cla)
    }

    /// Copies project to a target directory and yields the corresponding target project.
    ///
    /// - Recursively creates the target directory if needed.
    /// - Deletes any and all tla/cfg files not present in `self`.
    /// - Only copies files that either don't exist in the target, or are older in the target.
    pub fn to_target(&self, target: impl Into<io::PathBuf>, release: bool) -> Res<TargetProject> {
        let target = target.into();
        if !target.is_dir() {
            log::trace!("creating target directory `{}`", target.display());
            io::create_dir_all(&target).with_context(|| {
                anyhow!(
                    "failed to recursively create build directory `{}`",
                    target.display()
                )
            })?;
        }

        'remove_inexistent_targets: for entry in target
            .read_dir()
            .with_context(|| anyhow!("failed to read directory `{}`", target.display()))?
        {
            let entry = entry.with_context(|| {
                anyhow!(
                    "failed to read an entry of directory `{}`",
                    target.display()
                )
            })?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                continue 'remove_inexistent_targets;
            }

            let tla_or_cfg = io::file_ext(&entry_path)
                .map(|ext| ext == TlaFile::TLA_FILE_EXT || ext == TlaFile::CFG_FILE_EXT)
                .unwrap_or(false);
            if tla_or_cfg {
                let name = io::file_name(&entry_path)?;
                for file in self.files.iter() {
                    if io::file_name(file.path())? == name {
                        continue 'remove_inexistent_targets;
                    }
                }
                log::trace!("deleting target file `{}`", entry_path.display());
                io::remove_file(&entry_path)
                    .with_context(|| anyhow!("failed to delete file `{}`", entry_path.display()))?
            }
        }

        'copy_new_or_newer: for file in &self.files {
            let file_target = {
                let mut path = target.clone();
                path.push(io::file_name(file.path())?);
                path
            };
            if file_target.is_dir() {
                bail!("target file `{}` is a directory", file_target.display());
            }

            if file_target.is_file() {
                if io::file_modified(&file_target)? >= io::file_modified(file.path())? {
                    continue 'copy_new_or_newer;
                }
            }

            let _ = io::copy(file.path(), &file_target).with_context(|| {
                anyhow!(
                    "failed to copy `{}` to `{}`",
                    file.path().display(),
                    file_target.display(),
                )
            })?;
        }

        let target_project = self.change_path(&target)?;
        target_project.overwrite_matla_module(release)?;
        Ok(target_project)
    }
}

impl TargetProject {
    /// Overwrites the Matla module file, if it exists.
    ///
    /// - `release` controls whether the release or debug version of the module is written.
    ///
    /// Returns `true` if the existing Matla module file was overwritten, `false` otherwise (did
    /// nothing).
    pub fn overwrite_matla_module(&self, release: bool) -> Res<bool> {
        if let Some(file) = self.matla_module_file() {
            log::debug!(
                "writing {} `{}` module to `{}`",
                if release { "release" } else { "debug" },
                matla::MATLA_MODULE_NAME,
                file.path().display(),
            );
            let mut matla_module_file = io::write_file(file.path(), true, false)?;
            matla::write_module(&mut matla_module_file, release).with_context(|| {
                anyhow!(
                    "failed to write Matla module to `{}`",
                    file.path().display()
                )
            })?;
            Ok(true)
        } else {
            log::trace!("no `{}` module to write", matla::MATLA_MODULE_NAME);
            Ok(false)
        }
    }
}

/// A full project, aggregates the source [`Project`] and the target [`Project`].
#[derive(Debug, Clone)]
pub struct FullProject {
    /// Entry-point module, if any.
    pub entry: Option<String>,
    /// Actual entry point, either [`Self::entry`] if its `Some(_)`, or the unique runnable module.
    pub actual_entry: String,
    /// Target configuration.
    target_conf: conf::Target,
    /// Project's toml config file.
    pub config: conf::Project,
    /// Final TLC CLAs.
    pub tlc_cla: conf::TlcCla,
    /// Source project, the user's files.
    pub source: SourceProject,
    /// Target project, where we actually build/test/document stuff.
    pub target: TargetProject,
}
impl FullProject {
    /// Constructor.
    fn new(
        entry: Option<String>,
        target_conf: conf::Target,
        source: SourceProject,
        config: conf::Project,
        target: TargetProject,
        tlc_cla: Option<&conf::customl::TlcCla>,
    ) -> Res<(Self, conf::customl::TlcCla)> {
        let actual_entry = source
            .validate_runnable_module(entry.as_ref().map(|s| s as &str))?
            .into();
        let tlc_cla = {
            let mut res = conf::toolchain::user_read(|chain| chain.tlc_cla.clone())?;
            // println!("user TLC CLA:\n{:#?}", res);
            // println!("merging with project TLC CLA\n{:#?}", config.tlc_cla);
            res.receive(&config.tlc_cla);
            // println!("result:\n{:#?}", res);
            if let Some(tlc_cla) = tlc_cla {
                // println!("merging with call options\n{:#?}", tlc_cla);
                res.receive(&tlc_cla);
                // println!("receives top-level TLC CLA:\n{:#?}", res);
            }
            res
        };
        Ok((
            Self {
                entry,
                actual_entry,
                target_conf,
                source,
                config,
                target,
                tlc_cla: tlc_cla.clone().into(),
            },
            tlc_cla,
        ))
    }

    /// Source TLA file from a module name.
    pub fn source_tla_file(&self, module: impl AsRef<str>) -> Option<&TlaFile> {
        let module = module.as_ref();
        self.source
            .top_modules
            .get(module)
            .cloned()
            .map(|idx| &self.source[idx])
    }
    /// Target TLA file from a module name.
    pub fn target_tla_file(&self, module: impl AsRef<str>) -> Option<&TlaFile> {
        let module = module.as_ref();
        self.target
            .top_modules
            .get(module)
            .cloned()
            .map(|idx| &self.target[idx])
    }

    /// Generates a full TLC command taking into account user/project/CLA config (no module passed).
    pub fn tlc_cmd(&self) -> Res<io::Command> {
        self.target_conf.tlc_cmd(&self.tlc_cla)
    }

    /// Generates a full TLC command taking into account user/project/CLA (with module to run).
    pub fn full_tlc_cmd(&self, tool: bool) -> Res<io::Command> {
        let mut tlc_cmd = self.target_conf.custom_tlc_cmd(&self.tlc_cla, tool)?;
        let module = &self.actual_entry;
        if let Some(idx) = self.target.top_modules.get(module) {
            if !self.target.tla_to_cfg.contains_key(idx) {
                bail!(
                    "cannot run TLC on module `{}`: no cfg file associated to this module",
                    module,
                )
            }
            tlc_cmd.arg(self.target.files[*idx].file_name()?);
            Ok(tlc_cmd)
        } else {
            bail!("cannot run TLC on unknown module `{}`", module)
        }
    }

    /// Runs TLC on a module.
    pub fn run_tlc(&self) -> Res<io::Output> {
        let tlc_cmd = self.tlc_cmd()?;
        let module = &self.actual_entry;
        if let Some(idx) = self.target.top_modules.get(module) {
            if !self.target.tla_to_cfg.contains_key(idx) {
                bail!(
                    "cannot run TLC on module `{}`: no cfg file associated to this module",
                    module,
                )
            }
            self.target.files[*idx].run_tlc(tlc_cmd)
        } else {
            bail!("cannot run TLC on unknown module `{}`", module)
        }
    }

    /// Runs TLC on a module, async version.
    pub fn run_tlc_async<Out: tlc::Out>(&self, handler: Out) -> Res<tlc::TlcRun<Out>> {
        let tlc_cmd = self.tlc_cmd()?;
        let module = &self.actual_entry;
        if let Some(idx) = self.target.top_modules.get(module) {
            if !self.target.tla_to_cfg.contains_key(idx) {
                bail!(
                    "cannot run TLC on module `{}`: no cfg file associated to this module",
                    module,
                )
            }
            self.target.files[*idx].run_tlc_async(tlc_cmd, handler)
        } else {
            bail!("cannot run TLC on unknown module `{}`", module)
        }
    }

    /// Completes a TLC command.
    pub fn complete_tlc_cmd(&self) -> Res<io::Command> {
        let tlc_cmd = self.tlc_cmd()?;
        let module = &self.actual_entry;
        if let Some(idx) = self.target.top_modules.get(module) {
            if !self.target.tla_to_cfg.contains_key(idx) {
                bail!(
                    "cannot run TLC on module `{}`: no cfg file associated to this module",
                    module,
                )
            }
            self.target.files[*idx].complete_tlc_cmd(tlc_cmd)
        } else {
            bail!("cannot run TLC on unknown module `{}`", module)
        }
    }

    /// Loads the content of a module.
    pub fn load_module(&self, module: impl AsRef<str>, buf: &mut String) -> Res<()> {
        let module = module.as_ref();
        let idx = self
            .target
            .top_modules
            .get(module)
            .cloned()
            .ok_or_else(|| anyhow!("cannot load unknown module `{}`", module))?;
        let path = &self.target.files[idx].path;
        io::load_file_to(path, buf)
    }

    /// Loads the content of a module.
    pub fn module_content(&self, module: impl AsRef<str>) -> Res<String> {
        let mut buf = String::with_capacity(666);
        self.load_module(module, &mut buf)?;
        buf.shrink_to_fit();
        Ok(buf)
    }
}
