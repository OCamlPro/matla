//! Stores and handles the configuration.
//!
//! The content of this module should not be visible outside of this crate.

prelude!();

lazy_static! {
    /// Global variable for the configuration.
    pub(crate) static ref CONF: sync::RwLock<Option<Conf>>
    = sync::RwLock::new(None);
    /// Global variable for the project's configuration.
    pub(crate) static ref PROJECT_CONF: sync::RwLock<Option<project::Project>>
    = sync::RwLock::new(None);
    /// Top-level CLAP.
    pub(crate) static ref TOP_CLAP: sync::RwLock<Option<TopCla>>
    = sync::RwLock::new(None);
}

pub fn project_conf_read<Out>(
    action: impl FnOnce(Option<&project::Project>) -> Res<Out>,
) -> Res<Out> {
    let glob = PROJECT_CONF
        .read()
        .map_err(|_| anyhow!("[read] `PROJECT_CONF` lock is poisoned"))?;
    action(glob.as_ref())
}
pub fn project_conf_write<Out>(
    action: impl FnOnce(&mut Option<project::Project>) -> Res<Out>,
) -> Res<Out> {
    let mut glob = PROJECT_CONF
        .write()
        .map_err(|_| anyhow!("[write] `PROJECT_CONF` lock is poisoned"))?;
    action(&mut glob)
}

/// Reads the global configuration.
///
/// Fails if
/// - the configuration lock is poisoned.
pub fn read<'a>() -> sync::ReadRes<'a, Option<Conf>> {
    CONF.read()
        .map_err(|_| anyhow!("global configuration's lock is poisoned (on read access)"))
}
/// Fails if no global configuration is set, applies `action` otherwise.
///
/// Fails if
/// - the configuration is not set;
/// - the configuration lock is poisoned.
pub fn read_map<T>(action: impl FnOnce(&Conf) -> T) -> Res<T> {
    read().and_then(|opt| {
        let conf = opt
            .as_ref()
            .ok_or_else(|| anyhow!("cannot retrieve `tlc_path`, configuration is not set"))?;
        Ok(action(conf))
    })
}

/// Write access to the global configuration.
pub fn write<'a>() -> sync::WriteRes<'a, Option<Conf>> {
    CONF.write()
        .map_err(|_| anyhow!("global configuration's lock is poisoned (on write access)"))
}
/// Fails if no global configuration is set, applies `action` otherwise.
#[allow(dead_code)]
pub fn write_map<T>(action: impl FnOnce(&mut Conf) -> T) -> Res<T> {
    write().and_then(|mut opt| {
        let conf = opt
            .as_mut()
            .ok_or_else(|| anyhow!("cannot retrieve `tlc_path`, configuration is not set"))?;
        Ok(action(conf))
    })
}
