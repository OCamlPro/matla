//! Aggregates stuff from [`std::io`], [`std::fs`], and [`std::path`].

pub use std::{
    ffi::OsStr,
    fs::{copy, create_dir_all, remove_dir_all, remove_file, File, Metadata, OpenOptions},
    io::{BufRead, BufReader, Error, ErrorKind, Read, Result as Res, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output},
};

use crate::{anyhow, Context};

/// Tries to canonicalize a path.
///
/// On failure, [`log::error!`]s the error and returns the input path unchanged.
pub fn try_canonicalize(path: impl Into<PathBuf>, or_fail: bool) -> crate::Res<PathBuf> {
    let path = path.into();
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(e) => {
            let e = crate::Error::from(e)
                .context(format!("failed to canonicalize path `{}`", path.display(),));
            if or_fail {
                Err(e)
            } else {
                log::error!("Error handling path to matla binary:\n{:?}", e);
                Ok(path)
            }
        }
    }
}

/// Reads a line from `stdin`.
pub fn read_line() -> crate::Res<String> {
    let stdin = std::io::stdin();
    let mut buf = String::with_capacity(13);
    stdin
        .read_line(&mut buf)
        .context("failed to read from `stdin`")?;
    buf.shrink_to_fit();
    Ok(buf)
}

/// Asks a question to the user.
pub fn ask<T>(
    pref: &str,
    question: impl AsRef<str>,
    mut validator: impl FnMut(&str) -> Result<T, String>,
) -> crate::Res<T> {
    let question = question.as_ref();
    loop {
        println!("{}{}", pref, question);
        print!("{}", pref);
        std::io::stdout().flush().expect("failed to flush stdout");
        let answer = read_line()?;
        match validator(answer.trim()) {
            Ok(res) => return Ok(res),
            Err(e) => {
                println!("{}{}", pref, e);
                print!("{}", pref);
                std::io::stdout().flush().expect("failed to flush stdout");
            }
        }
    }
}
/// Asks a closed (yes/no) question to the user.
///
/// If `default_yes`, then empty answers are understood as *yes*, and *no* otherwise.
pub fn ask_closed(pref: &str, question: impl AsRef<str>, default_yes: bool) -> crate::Res<bool> {
    let mut question = question.as_ref().to_string();
    if default_yes {
        question.push_str(" [Yn]")
    } else {
        question.push_str(" [yN]")
    };
    ask(pref, question, |answer| match answer.as_ref() {
        "y" | "Y" | "yes" | "Yes" => Ok(true),
        "n" | "N" | "no" | "No" => Ok(false),
        "" => Ok(default_yes),
        _ => Err(format!(
            "unexpected answer `{}`, expected `y|Y|yes|Yes|n|N|no|No`",
            answer,
        )),
    })
}

/// Loads a file to a buffer.
pub fn load_file_to(path: impl AsRef<Path>, buf: &mut String) -> crate::Res<()> {
    let path = path.as_ref();
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| anyhow!("failed to load file `{}`", path.display()))?;
    let _ = file
        .read_to_string(buf)
        .with_context(|| anyhow!("failed to read content of file `{}`", path.display()))?;
    Ok(())
}
/// Loads a file.
pub fn load_file(path: impl AsRef<Path>) -> crate::Res<String> {
    let mut buf = String::with_capacity(113);
    load_file_to(path, &mut buf)?;
    Ok(buf)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn make_executable(_opts: &mut OpenOptions) {}
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn make_executable(opts: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    let mode = 0o770;
    opts.mode(mode);
}

/// Opens a file with write access.
pub fn write_file(path: impl AsRef<Path>, overwrite: bool, executable: bool) -> crate::Res<File> {
    let path = path.as_ref();
    let mut opts = OpenOptions::new();
    opts.write(true);
    if overwrite {
        opts.truncate(true).create(true);
    } else {
        opts.create_new(true);
    }
    if executable {
        make_executable(&mut opts);
    }
    opts.open(path).with_context(|| {
        anyhow!(
            "failed to open `{}` in write mode (overwrite: {})",
            path.display(),
            overwrite
        )
    })
}

/// Downloads something.
pub fn download(url: impl AsRef<str>) -> crate::Res<reqwest::blocking::Response> {
    let url = url.as_ref();
    reqwest::blocking::get(url).with_context(|| anyhow!("downloading `{}`", url))
}

/// Retrieves the file extension of a file path.
pub fn file_ext(path: impl AsRef<Path>) -> Option<std::ffi::OsString> {
    let path = path.as_ref();
    path.extension().map(std::ffi::OsString::from)
}

/// Retrieves the stem of a file path.
pub fn file_stem(path: impl AsRef<Path>) -> crate::Res<String> {
    let path = path.as_ref();
    Ok(path
        .file_stem()
        .ok_or_else(|| anyhow!("failed to retrieve file stem of `{}`", path.display()))?
        .to_string_lossy()
        .into_owned())
}

/// Retrieves the file name of a file path.
pub fn file_name(path: impl AsRef<Path>) -> crate::Res<String> {
    let path = path.as_ref();
    Ok(path
        .file_name()
        .ok_or_else(|| anyhow!("failed to retrieve file name of `{}`", path.display()))?
        .to_string_lossy()
        .into_owned())
}

/// Retrieves the metadata of a file path.
pub fn file_meta(path: impl AsRef<Path>) -> crate::Res<Metadata> {
    let path = path.as_ref();
    path.metadata()
        .with_context(|| anyhow!("failed to retrieve file metadata of `{}`", path.display()))
}
/// Retrieves the date of last modification of a file path.
pub fn file_modified(path: impl AsRef<Path>) -> crate::Res<crate::time::SystemTime> {
    let path = path.as_ref();
    let meta = file_meta(path)?;
    meta.modified().with_context(|| {
        anyhow!(
            "failed to retrieve date of last modification of `{}`",
            path.display(),
        )
    })
}
