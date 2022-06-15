//! Project-level configuration.

prelude!();

/// Name of the toml config file of a project.
pub const TOML_CONFIG_FILENAME: &str = "Matla.toml";

/// Applies some action to the global project configuration.
pub fn read<Out>(action: impl FnOnce(&Project) -> Out) -> Option<Out> {
    let top_clap = glob::PROJECT_CONF
        .read()
        .expect("[fatal] `PROJECT_CONF` global: lock poisoned");
    top_clap.as_ref().map(action)
}
/// Applies some action to the global project configuration.
pub fn try_read<Out>(action: impl FnOnce(&Project) -> Out) -> Res<Out> {
    read(action)
        .ok_or_else(|| anyhow!("[fatal] trying to ref-access `PROJECT_CONF`, but it is not set"))
}
/// Applies some action to the global project configuration.
pub fn write<Out>(action: impl FnOnce(&mut Project) -> Out) -> Option<Out> {
    let mut top_clap = glob::PROJECT_CONF
        .write()
        .expect("[fatal] `PROJECT_CONF` global: lock poisoned");
    top_clap.as_mut().map(action)
}
/// Applies some action to the global project configuration.
pub fn try_write<Out>(action: impl FnOnce(&mut Project) -> Out) -> Res<Out> {
    write(action)
        .ok_or_else(|| anyhow!("[fatal] trying to mut-access `PROJECT_CONF`, but it is not set"))
}

/// Loads a project configuration file.
///
/// Be careful that this function does **NOT** check that
/// - that the path exists, and
/// - is not a directory.
///
/// Please deal with these problems upstream.
pub fn raw_load(path: impl AsRef<io::Path>) -> Res<Project> {
    let path = path.as_ref();
    let mut content = io::OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| {
            anyhow!(
                "failed to load project configuration file `{}`",
                path.display(),
            )
        })?;
    let mut buf = String::with_capacity(666);
    {
        use io::Read;
        content.read_to_string(&mut buf).with_context(|| {
            anyhow!(
                "failed to read project configuration file `{}`",
                path.display(),
            )
        })?;
    }
    Project::de_toml(&buf)
}
/// Loads the project configuration from some path.
pub fn load_from(path: impl Into<io::PathBuf>) -> Res<bool> {
    let mut path = path.into();
    path.push(TOML_CONFIG_FILENAME);
    if !path.exists() {
        Ok(false)
    } else if path.is_dir() {
        bail!("`{}` is a directory, it should be a file", path.display())
    } else {
        let conf = raw_load(path)?;
        // println!("user conf:\n{:#?}", conf);
        glob::project_conf_write(|target| {
            if target.is_some() {
                bail!("attempting to load project configuration twice")
            } else {
                *target = Some(conf);
                Ok(())
            }
        })?;
        Ok(true)
    }
}
/// Loads the project configuration from the top-CLA project path.
pub fn load() -> Res<bool> {
    let path = top_cla::project_path()?;
    load_from(path)
}
/// Retrieves the project configuration, if any.
pub fn apply<Out>(action: impl FnOnce(&Project) -> Res<Out>) -> Res<Option<Out>> {
    glob::project_conf_read(|target| {
        if let Some(target) = target.as_ref() {
            action(target).map(Some)
        } else {
            Ok(None)
        }
    })
}

/// Configuration corresponding to a project's toml file.
#[derive(Debug, Clone)]
pub struct Project {
    /// TLC command-line arguments.
    pub tlc_cla: customl::TlcCla,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            tlc_cla: customl::TlcCla::default(),
        }
    }
}
impl Project {
    /// Serializes itself to toml.
    pub fn ser_toml(&self, w: &mut impl io::Write) -> Res<()> {
        writeln!(w, "[project]")?;
        self.tlc_cla.ser_toml_file(w)?;
        writeln!(w)?;
        Ok(())
    }
    /// Deserialization from toml.
    pub fn de_toml(txt: &str) -> Res<Self> {
        let mut tlc_cla = customl::TlcCla::none();
        customl::parse::config::project(txt, &mut tlc_cla)
            .map_err(|e| Error::msg(e.to_string()))?;
        Ok(Self { tlc_cla })
    }

    /// Dumps itself in some directory to a file named [`TOML_CONFIG_FILENAME`].
    ///
    /// The flag returned has the same semantics as [`Self::dump_to_file`].
    pub fn dump_to_dir(&self, path: impl AsRef<io::Path>) -> Res<bool> {
        let mut path: io::PathBuf = path.as_ref().into();
        path.push(TOML_CONFIG_FILENAME);
        self.dump_to_file(path)
    }

    /// Dumps itself in some file.
    ///
    /// Returns
    /// - `true` if the config file was written, and
    /// - `false` if the config file exists and is legal, in which case this function does nothing.
    pub fn dump_to_file(&self, path: impl AsRef<io::Path>) -> Res<bool> {
        let path = path.as_ref();
        if path.exists() {
            // Target exists, make sure it's a file, attempt to load it, and early exit if it's
            // legal.
            if path.is_dir() {
                bail!(
                    "cannot create project configuration file at `{}` because it is a directory",
                    path.display(),
                )
            }

            if raw_load(path).is_ok() {
                log::debug!("project config file present and legal, not overwriting it");
                return Ok(false);
            }
        }
        let mut target = io::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| {
                anyhow!(
                    "failed to load project configuration file `{}`",
                    path.display(),
                )
            })?;

        self.ser_toml(&mut target)?;
        Ok(true)
    }
}
