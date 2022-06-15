//! Basic dependencies, types and helpers.

#![forbid(rustdoc::broken_intra_doc_links)]

pub use std::{
    borrow::Cow,
    collections::{BTreeMap as Map, BTreeSet as Set},
    fmt,
    marker::PhantomData,
    mem,
    sync::mpsc,
};

pub use ansi_term as ansi;
pub use chrono;
pub use indenter;
pub use log;
pub use peg;
pub use rayon;
pub use regex;
pub use safe_index;
pub use walkdir::WalkDir;

pub use ansi::{ANSIGenericString as AnsiStr, Color, Style};
pub use anyhow::{anyhow, bail, Context, Error, Result as Res};
pub use either::{Either, Left, Right};
pub use implem::implem;
pub use lazy_static::lazy_static;
pub use num::{traits::ToPrimitive, BigInt as Int, BigUint as Nat, One, Zero};
pub use readonly::make as readonly;
pub use regex::Regex;
pub use smallvec::{smallvec, SmallVec as SVec};

pub mod io;
pub mod source;
pub mod thread;

/// Imports from [`std::sync`].
pub mod sync {
    pub use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

    /// A [`RwLockReadGuard`] wrapped in a [`crate::Res`].
    pub type ReadRes<'a, T> = crate::Res<RwLockReadGuard<'a, T>>;
    /// A [`RwLockWriteGuard`] wrapped in a [`crate::Res`].
    pub type WriteRes<'a, T> = crate::Res<RwLockWriteGuard<'a, T>>;
}

/// Time module, relies mostly on [`chrono`].
pub mod time {
    pub use std::time::{Duration, Instant, SystemTime};

    pub fn chrono_duration_fmt(d: &chrono::Duration) -> String {
        let mut s = String::with_capacity(203);

        macro_rules! try_fmt {
            (
                $(
                    $sep:literal
                    => $([force: $true:expr])? $fun:ident $(|$backup_fun:ident)?
                    => $qual:literal
                ),*
                $(,)?
            ) => {
                $({
                    let num = d.$fun() $(.unwrap_or_else(|| d.$backup_fun()))? ;
                    if num > 0 $(|| $true)? {
                        if !s.is_empty() {
                            s.push_str($sep)
                        }
                        s.push_str(&format!(concat!("{}", $qual), num));
                    }
                })*
            };
        }

        try_fmt! {
            ""   => num_weeks => "weeks",
            ", " => num_days  => "days",
            ", " => num_hours => "h",
            ""   => [force:true] num_seconds => "",
            "."  => [force:true] num_milliseconds => "s",
        }

        s.shrink_to_fit();
        s
    }
}

/// Contains macros that generate messages.
pub mod msg {
    pub use crate::{child_msg as child, fatal_msg as fatal};
}

/// Pretty-prints a potentially large number.
///
/// Adds `_` to separate `10^(3n)` blocks.
///
/// # Examples
///
/// ```rust
/// # use base::pretty_usize;
/// assert_eq!(pretty_usize(0), "0");
/// ```
pub fn pretty_usize(n: usize) -> String {
    let mut s = n.to_string();
    let mut pref = s.chars().count() % 3;

    while s[pref..].len() > 3 {
        s.insert(pref + 3, '_');
        pref += 4;
    }

    s
}

/// Creates a fatal message.
#[macro_export]
macro_rules! fatal_msg {
    { $txt:literal } => {
        concat!("[fatal] ", $txt)
    };
    { $($fmt:tt)* } => {
        format!(
            concat!($crate::fatal_msg!(""), "{}"),
            format!($($fmt)*),
        )
    };
}
/// Creates a child message.
#[macro_export]
macro_rules! child_msg {
    { $txt:literal } => {
        concat!("[child] ", $txt)
    };
    { $($fmt:tt)* } => {
        format!(
            concat!($crate::child_msg!(""), "{}"),
            format_args!($($fmt)*),
        )
    };
}

/// Sanitizes a string with `\`-escaping.
pub fn unescape_string(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let mut chars = s.chars();
    let mut res = String::with_capacity(s.len());

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                None => res.push('\\'),
                Some('n') => res.push('\n'),
                Some('t') => {
                    res.push('\t')
                    // res.push_str("    ");
                }
                Some('\\') => res.push('\\'),
                Some('"') => res.push('"'),
                Some('\'') => res.push('\''),
                Some(c) => {
                    res.push('\\');
                    res.push(c);
                }
            },
            '\t' => res.push_str("    "),
            _ => res.push(c),
        }
    }

    // println!("unescape");
    // for line in s.lines() {
    //     println!("| {}", line);
    // }
    // println!("=>");
    // for line in res.lines() {
    //     println!("| {}", line);
    // }

    res.shrink_to_fit();
    res
}

// /// Creates a prelude.
// ///
// /// Takes a list of tokens corresponding to some items for the `prelude` module to create.
// ///
// /// - creates a public module `prelude` containing the list of tokens;
// /// - creates a `#[macro_export]` macro `prelude!` importing the `prelude` module.
// ///
// /// This macro should be called in the top-most module of the crate. If you want your module to be
// /// at `some::path`, pass `@(some::path)` to this macro: `new_prelude! { @(some::path::) your_input }`.
// #[macro_export]
// macro_rules! new_prelude {
//     { @($($path:tt)*) $($mod_items:tt)* } => {
//         /// This crate's prelude.
//         pub mod prelude {
//             $($mod_items)*
//         }
//         /// Imports this crate's prelude.
//         #[macro_export]
//         macro_rules! prelude {
//             {$$($$stuff:tt)*} => {
//                 use $$crate::$$($path)*prelude::{*, $$($$stuff)*};
//             };
//         }
//     };
//     { $($mod_items:tt)* } => {
//         $crate::new_prelude!(@() $($mod_items:tt)*)
//     };
// }

// #[macro_use]
// pub mod blah {
//     new_prelude! {
//         @(blah::)
//         pub use std::io;
//     }
// }

// prelude!();
