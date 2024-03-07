//! This module defines various utilities to represent the [`SigLevel`] for
//! pacman repositories, including the [`read_conf`] function to retrieve
//! configuration from `pacman.conf` via the cli `pacman-conf`.

use alpm::SigLevel;
use std::{ffi::OsStr, process::Command};

/// Reads pacman.conf via the cli `pacman-conf`
pub fn read_conf<I, S>(args: I) -> String
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
        .output()
        .expect("failed to call pacman-conf")
        .stdout;

    let out_string = String::from_utf8_lossy(&cmd_out).to_string();

    match out_string.strip_suffix('\n') {
        Some(x) => x.to_string(),
        None => out_string,
    }
}

/// Parses and updates the SigLevel from the cli `pacman-conf`
fn update_siglevel(siglevel: &str, original: SigLevel) -> SigLevel {
    // process_siglevel: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/conf.c
    // show_siglevel: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/pacman-conf.c

    let slset = |sl: SigLevel| (original | sl) & !SigLevel::USE_DEFAULT;
    let slunset = |sl: SigLevel| (original & !sl) & !SigLevel::USE_DEFAULT;

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
        &_ => original,
    }
}

/// Updates the SigLevel(s) recursively, from a multiline string
fn recurse_siglevels(siglevels: String, original: SigLevel) -> SigLevel {
    let mut sig_level = original;
    for level in siglevels.split_terminator('\n') {
        sig_level = update_siglevel(level, sig_level)
    }
    return sig_level;
}

/// Finds the default SigLevel from `pacman.conf`
pub fn default_siglevel() -> SigLevel {
    let siglevels = read_conf(["SigLevel"]);
    return recurse_siglevels(siglevels, SigLevel::USE_DEFAULT);
}

/// Finds the SigLevel of a repo
pub fn repo_siglevel(repo: &str, default: SigLevel) -> SigLevel {
    let siglevels = read_conf(["--repo=", &repo, "SigLevel"]);
    return recurse_siglevels(siglevels, default);
}
