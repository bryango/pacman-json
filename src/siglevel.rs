//! A module that defines various utilities to represent the [`SigLevel`] for
//! pacman repositories.
//!
//! Note that similar functionalities are already provided in the `alpm-utils`
//! crate. This is a minimal re-implementation of components necessary for
//! `pacjump`.

use crate::read_conf;
use alpm::SigLevel;

/// Parses and updates a _single_ `SigLevel` line from the cli `pacman-conf`.
///
/// * `default`: [`SigLevel`] - the default [`SigLevel`] which would be
///                             stacked onto by the newly parsed `siglevel`
/// * `siglevel`: `&str` - the siglevel string to parse into a [`SigLevel`]
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
/// - `process_siglevel`: <https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/conf.c>
/// - `show_siglevel`: <https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/pacman-conf.c>
///
/// ### Examples:
///
/// ```
/// # use alpm::SigLevel;
/// # use pacjump::siglevel::process_siglevel;
/// #
/// // when an empty string is passed, the siglevel is unmodified;
/// // whitespace is ignored:
/// let siglevel = process_siglevel(SigLevel::PACKAGE_OPTIONAL, "\n\t \n");
/// assert_eq!(siglevel, SigLevel::PACKAGE_OPTIONAL);
///
/// // demands that packages require a signature:
/// let siglevel = process_siglevel(SigLevel::USE_DEFAULT, "PackageRequired");
/// assert_eq!(siglevel, SigLevel::PACKAGE);
/// ```
///
/// An unrecognized siglevel string would panic:
///
/// ```should_panic
/// # use alpm::SigLevel;
/// # use pacjump::siglevel::process_siglevel;
/// process_siglevel(SigLevel::USE_DEFAULT, "NonExistentSigLevel"); // panic!
///
/// // only fine-grained SigLevels are allowed:
/// process_siglevel(SigLevel::USE_DEFAULT, "Required"); // panic!
/// ```
///
pub fn process_siglevel(default: SigLevel, siglevel: &str) -> SigLevel {
    let slset = |sl: SigLevel| (default | sl) & !SigLevel::USE_DEFAULT;
    let slunset = |sl: SigLevel| (default & !sl) & !SigLevel::USE_DEFAULT;

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
        "" => default,
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
/// let siglevel = fold_siglevels(Sig::USE_DEFAULT, siglevel);
/// assert_eq!(siglevel, Sig::PACKAGE | Sig::DATABASE | Sig::DATABASE_OPTIONAL);
///
/// // empty lines are ignored:
/// let siglevel = fold_siglevels(Sig::DATABASE_OPTIONAL, "\n\n\n".into());
/// assert_eq!(siglevel, Sig::DATABASE_OPTIONAL);
///
/// let siglevel = fold_siglevels(Sig::DATABASE_OPTIONAL, "".into());
/// assert_eq!(siglevel, Sig::DATABASE_OPTIONAL);
/// ```
///
pub fn fold_siglevels(default: SigLevel, siglevels: String) -> SigLevel {
    siglevels
        .split_terminator('\n')
        .fold(default, process_siglevel)
}

/// Finds the default SigLevel from `pacman.conf`; if it fails, fall back to
/// the default [`SigLevel::USE_DEFAULT`].
pub fn default_siglevel() -> SigLevel {
    let siglevels = read_conf(["SigLevel"]).unwrap_or("".into());
    fold_siglevels(SigLevel::USE_DEFAULT, siglevels)
}

/// Finds the SigLevel of a repo; if it fails, fall back to the `default`.
pub fn repo_siglevel(repo: &str, default: SigLevel) -> SigLevel {
    let siglevels = read_conf(["--repo=", &repo, "SigLevel"]).unwrap_or("".into());
    fold_siglevels(default, siglevels)
}
