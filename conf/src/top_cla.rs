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
    pub fn new(color: bool, portable: bool, project_path: impl Into<io::PathBuf>) -> Self {
        Self {
            color,
            portable,
            log_level: log::LevelFilter::Warn,
            verb_level: 1,
            project_path: project_path.into(),
        }
    }

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

/// Explains what the `verb_level` configuration item does.
pub const VERB_LEVEL_DESC: &str = "\
Levels are cumulative:
- `0`: final result only;
- `1`: *absolute* state statistics, *i.e.* initial state count and final state count;
- `2`: time statistics;
- `3`: (virtually) all TLC statistics;
- `4`: almost all TLC info.
";

lazy_static! {
    /// Lazy-static for `verb_level`.
    ///
    /// This can be accessed often in practice, having a lazy-static avoids going through the
    /// [`sync::RwLock`] each time.
    pub static ref VERB_LEVEL: usize = try_read(|top| top.verb_level)
        .expect("[fatal] trying to access `VERB_LEVEL` before it is set");
}

/// A `println` conditionned by a verb level.
#[macro_export]
macro_rules! vlog {
    (if result $($tail:tt)*) => {
        $crate::vlog!( if (0) $($tail)* )
    };
    (if state stats $($tail:tt)*) => {
        $crate::vlog!( if (1) $($tail)* )
    };
    (if time stats $($tail:tt)*) => {
        $crate::vlog!( if (2) $($tail)* )
    };
    (if stats $($tail:tt)*) => {
        $crate::vlog!( if (3) $($tail)* )
    };
    (if max $($tail:tt)*) => {
        $crate::vlog!( if (4) $($tail)* )
    };

    ( if ($lvl:expr) $thn:block $(else $els:block)?  ) => {
        if $lvl == 0 || *$crate::top_cla::VERB_LEVEL >= $lvl
        $thn $(else $els)?
    };

    (result | $($interp_str:tt)*) => {
        $crate::vlog!( 0, $($interp_str)* )
    };
    (state stats | $($interp_str:tt)*) => {
        $crate::vlog!( 1, $($interp_str)* )
    };
    (time stats | $($interp_str:tt)*) => {
        $crate::vlog!( 2, $($interp_str)* )
    };
    (stats | $($interp_str:tt)*) => {
        $crate::vlog!( 3, $($interp_str)* )
    };
    (max | $($interp_str:tt)*) => {
        $crate::vlog!( 4, $($interp_str)* )
    };

    ( $lvl:expr, $($interp_str:tt)+ ) => {
        $crate::vlog!( if ($lvl) { println!($($interp_str)*) } )
    };
}

/// Accesses the top-level CLAP info verbosity argument.
pub fn verb_level() -> Res<usize> {
    try_read(|top| top.verb_level)
}
/// Sets the top-level CLAP verbosity argument.
pub fn set_verb_level(verb: usize) -> Res<()> {
    println!("setting verb level to {}", verb);
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
