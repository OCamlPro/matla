//! Toolchain configuration: provides helpers to generate commands.
//!
//! The [`Toolchain`] configuration is defined here but stored in the global [`Conf`]iguration.
//! Functions in this module such as [`tla2tools`] and [`tlc_cmd`] read the global configuration
//! and access the toolchain configuration to produce their output.
//!
//! As such, all these functions fail if the global configuration has not been set properly.
//!
//! The [`Toolchain`] configuration is created from the user's global configuration file, or from
//! the environment in portable mode.

prelude!();

/// Default `tla2tools` jar name.
pub const TLA2TOOLS_DEFAULT_NAME: &'static str = "tla2tools.jar";

/// Reads the toolchain setup from the user's configuration directory.
pub fn user_read<T>(action: impl FnOnce(&Toolchain) -> T) -> Res<T> {
    crate::glob::read_map(|conf| action(&conf.toolchain))
}

/// Path to the `tla2tools` jar.
///
/// Fails if the configuration is not loaded.
///
/// # Examples
///
/// ```rust
/// # use conf::toolchain::*;
/// conf::default_test_setup().unwrap();
/// conf::user::load().expect("failed to load user configuration");
///
/// let tla2tools = tla2tools().expect("unreachable");
/// assert!(tla2tools.exists());
/// assert!(tla2tools.is_file());
/// assert!( tla2tools.display().to_string().ends_with("tla2tools.jar") );
/// ```
pub fn tla2tools() -> Res<io::PathBuf> {
    crate::glob::read_map(|conf| conf.toolchain.tla2tools.clone())
}
/// TLC command.
///
/// Fails if the configuration is not loaded.
///
/// # Examples
///
/// ```rust
/// # use conf::toolchain::*;
/// conf::default_test_setup().unwrap();
/// conf::user::load().expect("failed to load user configuration");
///
/// let tlc_cmd = tlc_cmd().expect("unreachable");
/// assert_eq!(
///     format!("{:?}", tlc_cmd),
///     format!(
///         "{:?} {:?} {:?} {:?} {:?}",
///         "java",
///         "-XX:+UseParallelGC",
///         "-cp",
///         conf::toolchain::tla2tools().unwrap(),
///         "tlc2.TLC",
///     ),
/// );
/// ```
pub fn tlc_cmd() -> Res<io::Command> {
    crate::glob::read_map(|conf| conf.toolchain.tlc_cmd())
}
/// Documentation `tla2tex` command.
///
/// Fails if the configuration is not loaded.
///
/// # Examples
///
/// ```rust
/// # use conf::toolchain::*;
/// conf::default_test_setup().unwrap();
/// conf::user::load().expect("failed to load user configuration");
///
/// let tla2tex_cmd = tla2tex_cmd().expect("unreachable");
/// assert_eq!(
///     format!("{:?}", tla2tex_cmd),
///     format!(
///         "{:?} {:?} {:?} {:?} {:?}",
///         "java",
///         "-XX:+UseParallelGC",
///         "-cp",
///         conf::toolchain::tla2tools().unwrap(),
///         "tla2tex.TLA",
///     ),
/// );
/// ```
pub fn tla2tex_cmd() -> Res<io::Command> {
    crate::glob::read_map(|conf| conf.toolchain.tla2tex_cmd())
}

/// TLC options.
#[derive(Debug, Clone)]
pub struct Toolchain {
    /// Path to the `tla2tools.jar` binary.
    pub tla2tools: io::PathBuf,
    /// TLC CLA from the user's config toml file.
    pub tlc_cla: crate::customl::TlcCla,
}
impl Toolchain {
    /// Default `tla2tools` jar name.
    pub const TLA2TOOLS_DEFAULT_NAME: &'static str = TLA2TOOLS_DEFAULT_NAME;

    /// Serializes itself to TOML.
    pub fn ser_toml(&self, w: &mut impl io::Write) -> Res<()> {
        writeln!(
            w,
            "[config]\ntla2tools = '{}'\n\n\
            [tlc_cla]\n",
            self.tla2tools.display()
        )?;
        self.tlc_cla.ser_toml_file(w)?;
        writeln!(w)?;

        Ok(())
    }
    /// Deserializes itself from TOML.
    pub fn de_toml(txt: &str) -> Res<Self> {
        let mut tla2tools = io::PathBuf::new();
        let mut tlc_cla = crate::customl::TlcCla::none();
        customl::parse::config::user(txt, &mut tla2tools, &mut tlc_cla).map_err(Error::from)?;
        Ok(Self { tla2tools, tlc_cla })
    }

    /// Attempts to build a toolchain configuration from the environment.
    ///
    /// Will search for the [default tla2tools jar name][Self::TLA2TOOLS_DEFAULT_NAME].
    ///
    /// See also [`Self::from_env_with`].
    pub fn from_env() -> Res<Self> {
        Self::from_env_with(Self::TLA2TOOLS_DEFAULT_NAME)
    }
    /// Attempts to build a toolchain configuration from the environment and a custom command.
    ///
    /// See also [`Self::from_env`].
    pub fn from_env_with(tla2tools_cmd: impl AsRef<io::OsStr>) -> Res<Self> {
        let tla2tools_cmd = tla2tools_cmd.as_ref();
        let tla2tools = {
            let cmd_path: &io::Path = tla2tools_cmd.as_ref();
            if cmd_path.exists() {
                cmd_path.to_path_buf()
            } else {
                which::which(tla2tools_cmd).with_context(|| {
                    anyhow!(
                        "failed to retrieve path for `{}`",
                        tla2tools_cmd.to_string_lossy()
                    )
                })?
            }
        };

        Ok(Self {
            tla2tools,
            tlc_cla: crate::customl::TlcCla::default(),
        })
    }
    /// Sets a toolchain configuration as the global one.
    ///
    /// **Fails** if the global configuration is not set.
    ///
    /// Returns the previous configuration.
    pub fn register(self) -> Res<Toolchain> {
        crate::glob::write_map(|conf| mem::replace(&mut conf.toolchain, self))
    }

    /// Classless java call to [`Self::tla2tools`].
    fn java_cmd(&self) -> io::Command {
        let mut cmd = io::Command::new("java");
        cmd.args(["-XX:+UseParallelGC", "-cp"]).arg(&self.tla2tools);
        cmd
    }

    /// Mutable accessor to [`Self::tlc_cla`].
    pub fn tlc_cla_mut(&mut self) -> &mut crate::customl::TlcCla {
        &mut self.tlc_cla
    }

    /// Command for calling TLC.
    ///
    /// Users should use the [module-level `tlc_cmd` function][self::tlc_cmd], which calls this
    /// function on the global toolchain configuration.
    pub fn tlc_cmd(&self) -> io::Command {
        let mut cmd = self.java_cmd();
        cmd.arg("tlc2.TLC");
        cmd
    }
    /// Command for calling `tla2tex`.
    ///
    /// Users should use the [module-level `tla2tex_cmd` function][self::tla2tex_cmd], which calls
    /// this function on the global toolchain configuration.
    pub fn tla2tex_cmd(&self) -> io::Command {
        let mut cmd = self.java_cmd();
        cmd.arg("tla2tex.TLA");
        cmd
    }
}
implem! {
    for Toolchain {
        Display {
            |&self, fmt| {
                let mut bytes: Vec<u8> = Vec::with_capacity(666);
                self.ser_toml(&mut bytes).expect("writing to String cannot fail");
                String::from_utf8_lossy(&bytes).fmt(fmt)
            }
        }
    }
}
