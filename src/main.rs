// https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c#L192
// https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs#L38

use alpm::{Alpm, PackageReason, SigLevel};
use std::{process::Command, ffi::OsStr};

/// Reads pacman.conf via the cli `pacman-conf`
fn read_conf<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cmd_out =
        Command::new("pacman-conf")
        .args(args)
        .output()
        .expect("failed to call pacman-conf")
        .stdout;

    return String::from_utf8_lossy(&cmd_out).to_string();
}

/// Parses and updates the SigLevel from the cli `pacman-conf`
fn update_siglevel(siglevel: &str, original: SigLevel) -> SigLevel {

    // process_siglevel: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/conf.c
    // show_siglevel: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/pacman-conf.c

    let slset = |sl: SigLevel| { original | sl };
    let slunset = |sl: SigLevel| { original & !sl };

    let package_trust_all =
        SigLevel::PACKAGE_MARGINAL_OK
      | SigLevel::PACKAGE_UNKNOWN_OK;

    let database_trust_all =
        SigLevel::DATABASE_MARGINAL_OK
      | SigLevel::DATABASE_UNKNOWN_OK;

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
        &_ => original
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
fn default_siglevel() -> SigLevel {
    let siglevels = read_conf([ "SigLevel" ]);
    return recurse_siglevels(siglevels, SigLevel::USE_DEFAULT);
}

/// Finds the SigLevel of a repo
fn repo_siglevel(repo: &str, default: SigLevel) -> SigLevel {
    let siglevels = read_conf([ "--repo=", &repo, "SigLevel" ]);
    return recurse_siglevels(siglevels, default);
}

fn main() {

    let root = read_conf([ "RootDir" ]);
    let db_path = read_conf([ "DBPath" ]);
    let all_repos = read_conf([ "--repo-list" ]);

    let handle = Alpm::new(root, db_path).unwrap();

    let default_siglevel = default_siglevel();
    for repo in all_repos.split_terminator('\n') {
        let sig_level = repo_siglevel(repo, default_siglevel);
        handle
            .register_syncdb(repo, sig_level)
            .unwrap();
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // search each database for packages matching the regex "linux-[a-z]" AND "headers"
        for pkg in db.search(["linux-[a-z]", "headers"].iter()).unwrap() {
            println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // look for a package named "pacman" in each databse
        // the database is implemented as a hashmap so this is faster than iterating
        if let Ok(pkg) = db.pkg("pacman") {
            println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // iterate through every package in the databse
        for pkg in db.pkgs() {
            // print only explititly intalled packages
            if pkg.reason() == PackageReason::Explicit {
                println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
            }
        }
    }

    // iterate through each database
    for db in handle.syncdbs() {
        // look for the base-devel group
        if let Ok(group) = db.group("base-devel") {
            // print each package in the group
            for pkg in group.packages() {
                println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
            }
        }
    }

    // find a package matching a dep
    let pkg = handle.syncdbs().find_satisfier("linux>3").unwrap();
    println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));

    // load the pacman package from disk instead of from database
    let pkg = handle
        .pkg_load(
            "tests/pacman-5.1.3-1-x86_64.pkg.tar.xz",
            true,
            SigLevel::USE_DEFAULT,
        )
        .unwrap();
    println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
}
