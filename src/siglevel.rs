//! This module defines various utilities to represent the [`SigLevel`] for
//! pacman repositories, including the [`read_conf`] function to retrieve
//! configuration from `pacman.conf` via the cli `pacman-conf`.

use alpm::SigLevel;
use std::{ffi::OsStr, process::Command};

/// Reads pacman.conf via the cli `pacman-conf`. The arguments are directly
/// passed into [`Command::args`] and the result is parsed into a [`String`].
pub fn read_conf<I, S>(args: I) -> Result<String, std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_out = Command::new("pacman-conf")
        .args(args)
        .env("LC_ALL", "C.UTF-8")
        .env("LANGUAGE", "C.UTF-8")
        /*
            ^ making sure that user locales do not interfere;
            see: https://sourceware.org/bugzilla/show_bug.cgi?id=16621
        */
        .output()?
        .stdout;

    let out_string = String::from_utf8_lossy(&cmd_out).to_string();

    let trimmed_string = match out_string.strip_suffix('\n') {
        Some(x) => x.to_string(),
        None => out_string,
    };

    Ok(trimmed_string)
}

/// Parses and updates a [`SigLevel`] from the cli `pacman-conf`.
///
/// * `siglevel`: [`str`] - the siglevel string to parse into a [`SigLevel`]
/// * `prev`: [`SigLevel`] - the previous [`SigLevel`] which would be
///                          stacked onto by the newly parsed `siglevel`
///
/// This is a re-implementation of the following pacman functions:
///
/// - `process_siglevel`: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/conf.c
/// - `show_siglevel`: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/pacman-conf.c
///
fn process_siglevel(siglevel: &str, prev: SigLevel) -> SigLevel {
    let slset = |sl: SigLevel| (prev | sl) & !SigLevel::USE_DEFAULT;
    let slunset = |sl: SigLevel| (prev & !sl) & !SigLevel::USE_DEFAULT;

    let package_trust_all = SigLevel::PACKAGE_MARGINAL_OK | SigLevel::PACKAGE_UNKNOWN_OK;
    let database_trust_all = SigLevel::DATABASE_MARGINAL_OK | SigLevel::DATABASE_UNKNOWN_OK;

    match siglevel {
        "PackageNever" => slunset(SigLevel::PACKAGE),
        "PackageOptional" => slset(SigLevel::PACKAGE | SigLevel::PACKAGE_OPTIONAL),
        "PackageRequired" => slset(SigLevel::PACKAGE) & !SigLevel::PACKAGE_OPTIONAL,
        "PackageTrustedOnly" => slunset(package_trust_all),
        "PackageTrustAll" => slset(package_trust_all),
        "DatabaseNever" => slunset(SigLevel::DATABASE),
        "DatabaseOptional" => slset(SigLevel::DATABASE | SigLevel::DATABASE_OPTIONAL),
        "DatabaseRequired" => slset(SigLevel::DATABASE) & !SigLevel::DATABASE_OPTIONAL,
        "DatabaseTrustedOnly" => slunset(database_trust_all),
        "DatabaseTrustAll" => slset(database_trust_all),
        &_ => prev,
    }
}

/// Updates the [`SigLevel`] recursively, from a multiline string
fn fold_siglevels(siglevels: String, default: SigLevel) -> SigLevel {
    siglevels
        .split_terminator('\n')
        .fold(default, |prev, new| process_siglevel(new, prev))
}

/// Finds the default SigLevel from `pacman.conf`; if it fails, fall back to
/// the default [`SigLevel::USE_DEFAULT`].
pub fn default_siglevel() -> SigLevel {
    let siglevels = read_conf(["SigLevel"]).unwrap_or("".into());
    fold_siglevels(siglevels, SigLevel::USE_DEFAULT)
}

/// Finds the SigLevel of a repo; if it fails, fall back to the `default`.
pub fn repo_siglevel(repo: &str, default: SigLevel) -> SigLevel {
    let siglevels = read_conf(["--repo=", &repo, "SigLevel"]).unwrap_or("".into());
    fold_siglevels(siglevels, default)
}
