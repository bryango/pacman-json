mod info;
mod siglevel;

#[allow(unused_imports)]
use crate::info::{add_local_info, add_sync_info, decode_keyid, PackageInfo};
use crate::siglevel::{default_siglevel, read_conf, repo_siglevel};

use alpm::{Alpm, Db, Package, PackageReason};

/// Locates a Package from the databases by its name, prioritizing packages
/// from the sync database. The returned package must have the same packager
/// as the input package. The function panics if the package is not found in
/// the sync database nor in the local database.
fn db_with_pkg<'a>(handle: &'a Alpm, package: Package) -> Result<(Db<'a>, Package<'a>), String> {
    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs
    // dump_pkg_search, print_installed: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c
    // display, filter, pkg_get_locality: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/query.c

    let find_in_db = |db: Db<'a>| {
        // look for a package by name in a database; the database is
        // implemented as a hashmap so this is faster than iterating:
        if let Ok(pkg) = db.pkg(package.name()) {
            // verify that they share the same packager; we do not check
            // `version`, because the `local` version could be outdated
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
            return Ok((db, pkg));
        }
    }

    // otherwise, the package must be in the `local` database
    if let Some(pkg) = find_in_db(handle.localdb()) {
        return Ok((handle.localdb(), pkg));
    }
    Err(format!("{:?} not found in the databases", package))
}

/// Enriches local package with sync database information, if possible.
/// If the sync database information is available, it will be used as the
/// base package as it contains more information.
fn local_pkg_with_sync_info<'a>(handle: &'a Alpm, local_pkg: Package<'a>) -> PackageInfo<'a> {
    let local_info = PackageInfo::from(&local_pkg);

    let sync_pkg = match db_with_pkg(&handle, local_pkg) {
        Err(msg) => {
            eprintln!("{}", msg);
            return local_info;
        }
        Ok((_, x)) => x,
    };

    let sync_info = decode_keyid(&handle, PackageInfo::from(&sync_pkg));
    return add_local_info(local_info, sync_info);
    // return add_sync_info(local_info, sync_info); // alternatively
}

/// Dumps json data of the explicitly installed pacman packages.
/// Local packages are matched against the sync databases,
/// and upstream info is added to the output.
fn main() {
    let root = read_conf(["RootDir"]);
    let db_path = read_conf(["DBPath"]);
    let all_repos = read_conf(["--repo-list"]);
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
        handle.register_syncdb(repo, sig_level).unwrap();
        eprintln!("{repo}: SigLevel::{sig_level:?}");
    }
    eprintln!("");

    let db_type = "sync";
    let db_list = match db_type {
        "local" => vec![handle.localdb()],
        "sync" => handle.syncdbs().iter().collect(),
        _ => panic!("database type must be either \"local\" or \"sync\""),
    };

    let pkg_filters = "none";
    let pkg_filter_map: for<'a> fn(&'a Alpm, Package<'a>) -> Option<PackageInfo<'a>> =
        match pkg_filters {
            "explicit" => |handle, local_pkg| {
                if local_pkg.reason() == PackageReason::Explicit {
                    return Some(local_pkg_with_sync_info(&handle, local_pkg));
                }
                return None;
            },
            _ => |_, pkg| Some(PackageInfo::from(&pkg)),
        };

    let all_packages: Vec<PackageInfo<'_>> = db_list
        .iter()
        .map(|db| {
            db.pkgs()
                .iter()
                .filter_map(|pkg| pkg_filter_map(&handle, pkg))
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect(); // flattened list of packages

    let json = serde_json::to_string(&all_packages).expect("failed serializing json");
    println!("{}", json);
}
