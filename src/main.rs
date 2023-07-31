// https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c#L192
// https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs#L38

mod siglevel;
use crate::siglevel::{read_conf, default_siglevel, repo_siglevel};

use alpm::{Alpm, PackageReason, Package, Db};

/// Locates a Package from the databases
fn db_with_pkg<'a>(handle: &'a Alpm, package: Package) -> (Db<'a>, Package<'a>) {

    let find_in_db = |db: Db<'a>| {
        // look for a package by name in a database
        // the database is implemented as a hashmap
        // so this is faster than iterating:
        if let Ok(pkg) = db.pkg(package.name()) {
            // demand that they share the same packager
            // we do not check `version`
            // because the `local` version could be outdated
            if pkg.packager() == package.packager() {
                return Some(pkg);
            }
        }
        return None;
    };

    // iterate through each database
    for db in handle.syncdbs() {
        if let Some(pkg) = find_in_db(db) {
            return (db, pkg);
        }
    }

    // otherwise, the package must be in the `local` database
    if let Some(pkg) = find_in_db(handle.localdb()) {
        return (handle.localdb(), pkg);
    }
    panic!("{:?} not found in the databases", package)

}

fn main() {

    let root = read_conf([ "RootDir" ]);
    let db_path = read_conf([ "DBPath" ]);
    let all_repos = read_conf([ "--repo-list" ]);
    eprintln!("RootDir: {root}");
    eprintln!("DBPath: {db_path}");

    let default_siglevel = default_siglevel();
    eprintln!("SigLevel::{default_siglevel:?}");
    eprintln!("");

    let handle = Alpm::new(root, db_path).unwrap();

    // register sync databases from pacman.conf
    for repo in all_repos.split_terminator('\n') {
        let sig_level = repo_siglevel(repo, default_siglevel);
        handle
            .register_syncdb(repo, sig_level)
            .unwrap();
        // eprintln!("{repo}: SigLevel::{sig_level:?}");
    }

    eprintln!("Explicit:");
    for package in handle.localdb().pkgs() {
        if package.reason() == PackageReason::Explicit {
            let (db, pkg) = db_with_pkg(&handle, package);
            println!("{}/{} {}", db.name(), pkg.name(), pkg.packager().unwrap());
        }
    }

    // dump_pkg_search: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c

    // // iterate through each database
    // for db in handle.syncdbs() {
    //     // search each database for packages matching the regex "linux-[a-z]" AND "headers"
    //     for pkg in db.search(["linux-[a-z]", "headers"].iter()).unwrap() {
    //         println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
    //     }
    // }

    // // iterate through each database
    // for db in handle.syncdbs() {
    //     // look for a package named "pacman" in each databse
    //     // the database is implemented as a hashmap so this is faster than iterating
    //     if let Ok(pkg) = db.pkg("pacman") {
    //         println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
    //     }
    // }

    // // iterate through each database
    // for db in handle.syncdbs() {
    //     // iterate through every package in the databse
    //     for pkg in db.pkgs() {
    //         // print only explititly intalled packages
    //         if pkg.reason() == PackageReason::Explicit {
    //             println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
    //         }
    //     }
    // }

    // // iterate through each database
    // for db in handle.syncdbs() {
    //     // look for the base-devel group
    //     if let Ok(group) = db.group("base-devel") {
    //         // print each package in the group
    //         for pkg in group.packages() {
    //             println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
    //         }
    //     }
    // }

    // // find a package matching a dep
    // let pkg = handle.syncdbs().find_satisfier("linux>3").unwrap();
    // println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));

    // // load the pacman package from disk instead of from database
    // let pkg = handle
    //     .pkg_load(
    //         "tests/pacman-5.1.3-1-x86_64.pkg.tar.xz",
    //         true,
    //         SigLevel::USE_DEFAULT,
    //     )
    //     .unwrap();
    // println!("{} {}", pkg.name(), pkg.desc().unwrap_or("None"));
}
