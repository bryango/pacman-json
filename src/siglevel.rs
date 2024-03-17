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

/// Parses and updates a _single_ `SigLevel` line from the cli `pacman-conf`.
///
/// * `siglevel`: [`str`] - the siglevel &str to parse into a [`SigLevel`]
/// * `prev`: [`SigLevel`] - the previous [`SigLevel`] which would be
///                          stacked onto by the newly parsed `siglevel`
///
/// The `SigLevel` return of `pacman-conf` is a fine-grained subset of those
/// allowed in `pacman.conf`. For example, `SigLevel = Required` in
/// `pacman.conf` will be resolved to two SigLevel lines by `pacman-conf`:
/// one is `PackageRequired`, and the other is `DatabaseRequired`. We are only
/// committed to parsing this fined-grained subset returned from `pacman-conf`.
/// In particular, plain `Required` without the `Package` or `Database` prefix
/// would not be allowed, and the code will panic.
///
/// This is a re-implementation of the following pacman functions:
///
/// - `process_siglevel`: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/conf.c
/// - `show_siglevel`: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/pacman-conf.c
///
/// ### Examples:
///
/// ```
/// # use alpm::SigLevel;
/// # use pacjump::siglevel::process_siglevel;
/// #
/// // when an empty string is passed, the siglevel is unmodified;
/// // whitespace is ignored:
/// let siglevel = process_siglevel("\n\t \n", SigLevel::PACKAGE_OPTIONAL);
/// assert_eq!(siglevel, SigLevel::PACKAGE_OPTIONAL);
///
/// // demands that packages require a signature:
/// let siglevel = process_siglevel("PackageRequired", SigLevel::USE_DEFAULT);
/// assert_eq!(siglevel, SigLevel::PACKAGE);
/// ```
///
/// An unrecognized siglevel string would panic:
///
/// ```should_panic
/// # use alpm::SigLevel;
/// # use pacjump::siglevel::process_siglevel;
/// process_siglevel("NonExistentSigLevel", SigLevel::USE_DEFAULT); // panic!
///
/// // only fine-grained SigLevels are allowed:
/// process_siglevel("Required", SigLevel::USE_DEFAULT); // panic!
/// ```
///
pub fn process_siglevel(siglevel: &str, prev: SigLevel) -> SigLevel {
    let slset = |sl: SigLevel| (prev | sl) & !SigLevel::USE_DEFAULT;
    let slunset = |sl: SigLevel| (prev & !sl) & !SigLevel::USE_DEFAULT;

    let package_trust_all = SigLevel::PACKAGE_MARGINAL_OK | SigLevel::PACKAGE_UNKNOWN_OK;
    let database_trust_all = SigLevel::DATABASE_MARGINAL_OK | SigLevel::DATABASE_UNKNOWN_OK;

    match siglevel.trim() {
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
        "" => prev,
        x => panic!("failed to parse the signature level: {}", x),
    }
}

/// Updates the [`SigLevel`] recursively, from a multiline string.
///
/// ### Examples:
///
/// ```
/// # use alpm::SigLevel;
/// # use pacjump::siglevel::fold_siglevels;
/// #
/// let siglevel: String = [
///     "PackageRequired",
///     "PackageTrustedOnly",
///     "DatabaseOptional",
///     "DatabaseTrustedOnly",
/// ].join("\n");
///
/// use SigLevel as Sig;
/// let siglevel = fold_siglevels(siglevel, Sig::USE_DEFAULT);
/// assert_eq!(siglevel, Sig::PACKAGE | Sig::DATABASE | Sig::DATABASE_OPTIONAL);
///
/// // empty lines are ignored:
/// let siglevel = fold_siglevels("\n\n\n".into(), Sig::DATABASE_OPTIONAL);
/// assert_eq!(siglevel, Sig::DATABASE_OPTIONAL);
///
/// let siglevel = fold_siglevels("".into(), Sig::DATABASE_OPTIONAL);
/// assert_eq!(siglevel, Sig::DATABASE_OPTIONAL);
/// ```
///
pub fn fold_siglevels(siglevels: String, default: SigLevel) -> SigLevel {
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
