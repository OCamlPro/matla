//! Handles user-configuration-related paths: creation, loading, etc.
//!
//! This module does not really have any use outside of this crate.

prelude!();

/// Applies some action to the global project configuration.
pub fn read<Out>(action: impl FnOnce(&Conf) -> Out) -> Option<Out> {
    let top_clap = glob::CONF
        .read()
        .expect("[fatal] `CONF` global: lock poisoned");
    top_clap.as_ref().map(action)
}
/// Applies some action to the global project configuration.
pub fn try_read<Out>(action: impl FnOnce(&Conf) -> Out) -> Res<Out> {
    read(action).ok_or_else(|| anyhow!("[fatal] trying to ref-access `CONF`, but it is not set"))
}
/// Applies some action to the global project configuration.
pub fn write<Out>(action: impl FnOnce(&mut Conf) -> Out) -> Option<Out> {
    let mut top_clap = glob::CONF
        .write()
        .expect("[fatal] `CONF` global: lock poisoned");
    top_clap.as_mut().map(action)
}
/// Applies some action to the global project configuration.
pub fn try_write<Out>(action: impl FnOnce(&mut Conf) -> Out) -> Res<Out> {
    write(action).ok_or_else(|| anyhow!("[fatal] trying to mut-access `CONF`, but it is not set"))
}

/// Path to the configuration directory.
///
/// Fails if the home directory cannot be retrieved.
///
/// # Examples
///
/// ```rust
/// # use conf::user::conf_path;
/// use path_slash::PathExt;
///
/// let conf_path = conf_path().unwrap();
/// let conf_path = conf_path.to_slash_lossy();
/// # println!("conf_path: {}", conf_path);
/// assert!(
///     conf_path.ends_with(&format!(
///         "{}/{}",
///         conf::user::CONF_DIR,
///         conf::user::MATLA_CONF_SUBDIR,
///     ))
/// );
/// ```
pub fn conf_path() -> Res<io::PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| anyhow!("failed to retrieve home directory"))?;
    path.push(CONF_DIR);
    path.push(MATLA_CONF_SUBDIR);
    Ok(path)
}

/// Path to the matla toml configuration file.
///
/// Fails if the home directory cannot be retrieved.
///
/// # Examples
///
/// ```rust
/// # use conf::user::toml_path;
/// use path_slash::PathExt;
///
/// let toml_path = toml_path().unwrap();
/// let toml_path = toml_path.to_slash_lossy();
/// # println!("toml_path: {}", toml_path);
/// assert!(
///     toml_path.ends_with(&format!(
///         "{}/{}/{}",
///         conf::user::CONF_DIR,
///         conf::user::MATLA_CONF_SUBDIR,
///         conf::user::MATLA_CONF_FILE,
///     ))
/// );
/// ```
pub fn toml_path() -> Res<io::PathBuf> {
    let mut toml_path = conf_path()?;
    toml_path.push(MATLA_CONF_FILE);
    Ok(toml_path)
}

/// Loads the user configuration file from some path.
///
/// Fails if
/// - toml configuration file is ill-formed.
pub fn raw_load(path: impl AsRef<io::Path>) -> Res<Conf> {
    let path = path.as_ref();
    log::trace!("loading user config at `{}`", path.display());
    let content = io::load_file(path).context("failed to load user configuration")?;
    Conf::de_toml(&content)
        .with_context(|| anyhow!("failed to parse configuration file `{}`", path.display()))
}

/// Loads the user configuration file from some path and registers the result.
///
/// Fails if
/// - toml configuration file is ill-formed.
pub fn load_from(path: impl AsRef<io::Path>) -> Res<()> {
    let conf = raw_load(path)?;
    let mut glob = glob::CONF
        .write()
        .expect("[fatal] `PROJECT_CONF` global: lock poisoned");
    *glob = Some(conf);
    Ok(())
}

/// Loads the user configuration file.
///
/// Fails if
/// - home directory cannot be retrieved;
/// - toml configuration file is ill-formed.
pub fn auto_load() -> Res<Conf> {
    let toml_path = toml_path()?;
    log::trace!("loading user config at `{}`", toml_path.display());
    let content = io::load_file(&toml_path).context("failed to load user configuration")?;
    Conf::de_toml(&content).with_context(|| {
        anyhow!(
            "failed to parse configuration file `{}`",
            toml_path.display()
        )
    })
}

/// Loads the configuration, portable mode.
pub fn load_portable() -> Res<()> {
    let conf = Conf::from_env()?;
    conf.register()?;
    Ok(())
}

/// Loads the configuration.
///
/// If `portable` (from [`top_cla`]), loads the configuration from the environment.
pub fn load() -> Res<()> {
    let portable = crate::top_cla::portable()?;
    if portable {
        load_portable()?;
    } else {
        let conf = auto_load()?;
        conf.register()?;
    }
    Ok(())
}

/// Configuration directory in user's home.
pub const CONF_DIR: &str = ".config";
/// Matla configuration subdirectory.
pub const MATLA_CONF_SUBDIR: &str = "matla";
/// Matla main configuration file.
pub const MATLA_CONF_FILE: &str = "matla.toml";
/// TLA toolbox jar file.
pub const TLA2TOOLS_FILE: &str = crate::toolchain::TLA2TOOLS_DEFAULT_NAME;

/// Path to the `tla2tools` jar in the user's config directory.
///
/// Fails if the home directory cannot be retrieved.
///
/// # Examples
///
/// ```rust
/// # use conf::user::tla2tools_jar_path;
/// use path_slash::PathExt;
///
/// let tla2tools_jar_path = tla2tools_jar_path().unwrap();
/// let tla2tools_jar_path = tla2tools_jar_path.to_slash_lossy();
/// # println!("tla2tools_jar_path: {}", tla2tools_jar_path);
/// assert!(
///     tla2tools_jar_path.ends_with(&format!(
///         "{}/{}/{}",
///         conf::user::CONF_DIR,
///         conf::user::MATLA_CONF_SUBDIR,
///         conf::user::TLA2TOOLS_FILE,
///     ))
/// );
/// ```
pub fn tla2tools_jar_path() -> Res<io::PathBuf> {
    let mut tla2tools = conf_path()?;
    tla2tools.push(TLA2TOOLS_FILE);
    Ok(tla2tools)
}

/// Writes to the user configuration file.
///
/// Creates the directories required, if any.
pub fn dump(conf: &Conf, overwrite: bool) -> Res<()> {
    let mut conf_path = conf_path()?;
    io::create_dir_all(&conf_path).with_context(|| {
        anyhow!(
            "failed to (recursively) create path `{}`",
            conf_path.display()
        )
    })?;
    let toml_path = {
        conf_path.push(MATLA_CONF_FILE);
        conf_path
    };
    log::trace!(
        "writing user config file at `{}` (overwrite: {})",
        toml_path.display(),
        overwrite
    );
    let mut file = io::write_file(&toml_path, overwrite, false).with_context(|| {
        anyhow!(
            "failed to write-load user configuration file `{}`",
            toml_path.display()
        )
    })?;

    conf.ser_toml(&mut file).with_context(|| {
        anyhow!(
            "failed to write to user configuration file `{}`",
            toml_path.display()
        )
    })?;

    Ok(())
}
