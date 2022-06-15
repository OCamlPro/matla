//! Testing-related errors.

prelude!();

/// Parsing error from `peg`.
pub type PegError = peg::error::ParseError<peg::str::LineCol>;
/// Result with a parsing error.
pub type PegRes<T> = Result<T, PegError>;

/// Result of loading an integration test.
pub type ITestRes<T> = Result<T, ITestLoadFailed>;

/// Integration test loading.
pub enum ITestLoadFailed {
    /// Trying to load a non-existent/-file TLA file.
    TlaNotAFile(io::PathBuf),
    /// `cfg` file non-existent/-file.
    CfgNotAFile(io::PathBuf),
    /// Illegal test header.
    IllegalConf(peg::error::ParseError<peg::str::LineCol>),
    /// Some other error.
    Other(base::Error),
}
