// https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c#L192
// https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs#L38

use alpm::{Alpm, PackageReason, SigLevel};
use std::{process::Command, ffi::OsStr};

/// Reads pacman.conf from the cli `pacman-conf`
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

fn main() {

    let root = read_conf([ "RootDir" ]);
    let db_path = read_conf([ "DBPath" ]);
    let repo_list = read_conf([ "--repo-list" ]);
    let handle = Alpm::new(root, db_path).unwrap();

    for repo in repo_list.split_terminator('\n') {
        let sig_level = SigLevel::USE_DEFAULT;
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
