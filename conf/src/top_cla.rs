//! Top-level CLA values.

prelude!();

/// Applies some action to the global top-level CLAP values.
pub fn read<Out>(action: impl FnOnce(&TopCla) -> Out) -> Option<Out> {
    let top_clap = glob::TOP_CLAP
        .read()
        .expect("[fatal] `TOP_CLAP` global: lock poisoned");
    top_clap.as_ref().map(action)
}
/// Applies some action to the global top-level CLAP values.
pub fn try_read<Out>(action: impl FnOnce(&TopCla) -> Out) -> Res<Out> {
    read(action)
        .ok_or_else(|| anyhow!("[fatal] trying to ref-access `TOP_CLAP`, but it is not set"))
}
/// Applies some action to the global top-level CLAP values.
pub fn write<Out>(action: impl FnOnce(&mut TopCla) -> Out) -> Option<Out> {
    let mut top_clap = glob::TOP_CLAP
        .write()
        .expect("[fatal] `TOP_CLAP` global: lock poisoned");
    top_clap.as_mut().map(action)
}
/// Applies some action to the global top-level CLAP values.
pub fn try_write<Out>(action: impl FnOnce(&mut TopCla) -> Out) -> Res<Out> {
    write(action)
        .ok_or_else(|| anyhow!("[fatal] trying to mut-access `TOP_CLAP`, but it is not set"))
}

/// Registers a default [`TopCla`], mostly used for tests.
pub fn register_test_version() -> Res<()> {
    TopCla::new_test().init()
}

/// Top-level CLAP values.
pub struct TopCla {
    /// Color.
    pub color: bool,
    /// Portable.
    pub portable: bool,
    /// Log level.
    pub log_level: log::LevelFilter,
    /// Verbosity level.
    pub verb_level: usize,
    /// Path to the project directory.
    pub project_path: io::PathBuf,
}
impl TopCla {
    /// Sets the top-level CLAP values.
    pub fn init(self) -> Res<()> {
        let mut top = glob::TOP_CLAP
            .write()
            .map_err(|_| anyhow!("`TOP_CLAP`'s lock is poisoned"))?;
        *top = Some(self);
        Ok(())
    }

    /// Test setup constructor.
    pub fn new_test() -> Self {
        Self {
            color: false,
            portable: false,
            log_level: log::LevelFilter::Trace,
            verb_level: 0,
            project_path: ".".into(),
        }
    }
}

/// Accesses the top-level CLAP info verbosity argument.
pub fn verb_level() -> Res<usize> {
    try_read(|top| top.verb_level)
}
/// Sets the top-level CLAP verbosity argument.
pub fn set_verb_level(verb: usize) -> Res<()> {
    try_write(|top| top.verb_level = verb)
}
/// Applies some action to the top-level CLAP verbosity argument.
pub fn verb_level_do(action: impl FnOnce(usize) -> usize) -> Res<()> {
    try_write(|top| top.verb_level = action(top.verb_level))
}

/// Accesses the top-level CLAP log verbosity argument.
pub fn log_level() -> Res<log::LevelFilter> {
    try_read(|top| top.log_level)
}
/// Sets the top-level CLAP log verbosity argument.
pub fn set_log_level(level: log::LevelFilter) -> Res<()> {
    try_write(|top| top.log_level = level)
}

/// Accesses the top-level CLAP color argument.
pub fn color() -> Res<bool> {
    try_read(|top| top.color)
}
/// Sets the top-level CLAP color argument.
pub fn set_color(color: bool) -> Res<()> {
    try_write(|top| top.color = color)
}

/// Accesses the top-level CLAP portable argument.
pub fn portable() -> Res<bool> {
    try_read(|top| top.portable)
}
/// Sets the top-level CLAP portable argument.
pub fn set_portable(portable: bool) -> Res<()> {
    try_write(|top| top.portable = portable)
}

/// Accesses the top-level CLAP project path argument.
pub fn project_path() -> Res<io::PathBuf> {
    try_read(|top| top.project_path.clone())
}
/// Sets the top-level CLAP project path argument.
pub fn set_project_path(project_path: impl Into<io::PathBuf>) -> Res<()> {
    try_write(|top| top.project_path = project_path.into())
}
