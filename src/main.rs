
mod siglevel;
mod info;

#[allow(unused_imports)]
use crate::info::{PackageInfo, decode_keyid, add_sync_info, add_local_info};
use crate::siglevel::{read_conf, default_siglevel, repo_siglevel};

use alpm::{Alpm, Package, Db, PackageReason};

/// Locates a Package from the databases
fn db_with_pkg<'a>(handle: &'a Alpm, package: Package) -> (Db<'a>, Package<'a>) {

    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs
    // dump_pkg_search, print_installed: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c
    // display, filter, pkg_get_locality: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/query.c

    let find_in_db = |db: Db<'a>| {
        // look for a package by name in a database
        // the database is implemented as a hashmap
        // so this is faster than iterating:
        if let Ok(pkg) = db.pkg(package.name()) {
            // demand that they share the same packager
            // we do not check `version`
            // because the `local` version could be outdated
            if pkg.packager() == package.packager() {
                // ^ Deref coercion for method call: Package -> Pkg
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

/// Dumps json data of the explicitly installed pacman packages.
/// Local packages are matched against the sync databases,
/// and upstream info is added to the output.
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
    eprintln!("--repo-list:");
    for repo in all_repos.split_terminator('\n') {
        let sig_level = repo_siglevel(repo, default_siglevel);
        handle
            .register_syncdb(repo, sig_level)
            .unwrap();
        eprintln!("{repo}: SigLevel::{sig_level:?}");
    }
    eprintln!("");

    // going through explicitly installed packages
    let explicits: Vec<PackageInfo> = Vec::from_iter(
        handle.localdb().pkgs().iter().filter_map(
            |local_pkg| {
                if local_pkg.reason() == PackageReason::Explicit {
                    let (_, sync_pkg) = db_with_pkg(&handle, local_pkg);
                    let local_info = PackageInfo::from(&local_pkg);
                    let sync_info = decode_keyid(
                        &handle,
                        PackageInfo::from(&sync_pkg)
                    );
                    return Some(add_local_info(local_info, sync_info));
                    // return Some(add_sync_info(local_info, sync_info));
                }
                return None;
            }
        )
    );

    let json = serde_json::to_string(&explicits)
        .expect("failed serializing json");
    println!("{}", json);

}
