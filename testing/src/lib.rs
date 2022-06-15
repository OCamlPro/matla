//! Handles TLA project testing.

#![forbid(rustdoc::broken_intra_doc_links)]

use project::FullProject;

/// This crate's prelude.
pub mod prelude {
    pub use base::*;
    pub use conf;
    pub use project::{self, tlc};

    pub use crate::{doc, err::*, integration, Filter, TestRes};
}
/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    {$($stuff:tt)*} => {
        use $crate::prelude::{*, $($stuff)*};
    };
}

prelude!();

pub mod doc;
pub mod err;
pub mod integration;

/// A list of regex-es.
#[derive(Debug, Clone)]
pub struct Filter {
    pub regexes: Vec<Regex>,
}
impl Filter {
    /// Constructor.
    pub fn new() -> Self {
        Self { regexes: vec![] }
    }
    /// Tests whether the input string is matched by a regex.
    pub fn contains(&self, s: impl AsRef<str>) -> bool {
        let s = s.as_ref();
        self.regexes.iter().any(|regex| {
            let res = regex.is_match(s);
            // println!("matching `{}` on `{}` ({})", regex, s, res);
            res
        })
    }

    /// Adds a regex.
    pub fn add(&mut self, s: &str) -> Res<()> {
        let regex = Regex::new(s).map_err(|e| {
            Error::msg(e.to_string()).context(anyhow!("failed to parle test filter regex `{}`", s))
        })?;
        self.regexes.push(regex);
        Ok(())
    }
}
// implem! {
//     for Filter {
//         Deref<Target = Vec<Regex>> {
//             |&self| &self.regexes,
//             |&mut self| &mut self.regexes,
//         }
//     }
// }

/// A documentation test.
pub type DocTest = Test<doc::Kind>;

/// Test result.
pub type TestRes = Result<(), Vec<String>>;

/// A test.
#[readonly]
pub struct Test<Kind> {
    /// Test kind.
    pub kind: Kind,
    /// Test full project.
    pub project: FullProject,
    /// Expected outcome.
    pub expected: tlc::RunOutcome,
}
impl<Src> Test<Src> {
    /// Constructor.
    pub fn new(kind: Src, project: FullProject, expected: tlc::RunOutcome) -> Self {
        Self {
            kind,
            project,
            expected,
        }
    }
}
