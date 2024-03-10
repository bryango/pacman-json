pub mod info;
pub mod reverse_deps;
pub mod siglevel;

use alpm::{Alpm, Package, PackageReason};
use clap::Parser;
use info::{add_local_info, add_sync_info, decode_keyid, PackageInfo};

/// This struct contains the available filters for pacman packages, exposed
/// through the command line interface.
#[derive(Debug, Parser)]
#[command(about)]
pub struct PackageFilters {
    /// Query the sync databases; by default we only query the local database
    /// with the currently installed packages.
    #[arg(long)]
    pub sync: bool,

    /// Query all packages, including those not explicitly installed;
    /// by default only explicitly installed packages are shown.
    #[arg(long)]
    pub all: bool,

    /// Output package info from the current database only; by default we
    /// enrich the output by combining information from both the local
    /// and the sync databases.
    #[arg(long)]
    pub plain: bool,
}

/// Applies an instance of [`PackageFilters`] to a [`alpm::Package`], and
/// optionally returns the desired [`info::PackageInfo`].
pub fn pkg_filter_map<'a>(
    handle: &'a Alpm,
    pkg: Package<'a>,
    pkg_filters: &PackageFilters,
) -> Option<PackageInfo<'a>> {
    if !pkg_filters.all && pkg.reason() != PackageReason::Explicit {
        return None;
    }
    if pkg_filters.plain {
        return Some(PackageInfo::from(&pkg));
    }
    return Some(enrich_pkg_info(&handle, pkg, &pkg_filters));
}

/// Enriches package with sync & local database information, if desired and
/// when possible. If the sync database information is available and accurate,
/// it will be preferred as the base info since it contains more details.
pub fn enrich_pkg_info<'a>(
    handle: &'a Alpm,
    pkg: Package<'a>,
    pkg_filters: &PackageFilters,
) -> PackageInfo<'a> {
    let base_info = PackageInfo::from(&pkg);

    if pkg_filters.sync {
        let sync_pkg = pkg;
        let sync_info = decode_keyid(&handle, base_info);
        if pkg_filters.plain {
            return sync_info;
        }
        match handle.localdb().pkg(sync_pkg.name()) {
            Err(_) => return sync_info,
            Ok(local_pkg) => {
                let local_info = PackageInfo::from(&local_pkg);
                return add_local_info(local_info, sync_info);
            }
        };
    }

    // otherwise, the input `pkg` is local:
    let local_pkg = pkg;
    let local_info = base_info;
    let sync_pkg = match find_in_syncdb(&handle, local_pkg) {
        Err(msg) => {
            eprintln!("{}", msg);
            return local_info;
        }
        Ok(x) => x,
    };
    let sync_info = decode_keyid(&handle, PackageInfo::from(&sync_pkg));

    return match pkg_filters.plain
        || local_pkg.packager() != sync_pkg.packager()
        || local_pkg.version() != sync_pkg.version()
    {
        true => add_sync_info(local_info, sync_info),
        false => add_local_info(local_info, sync_info),
    };
}

/// Locates a Package from the sync databases by its name.
pub fn find_in_syncdb<'a>(handle: &'a Alpm, package: Package) -> Result<Package<'a>, String> {
    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs
    // dump_pkg_search, print_installed: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c
    // display, filter, pkg_get_locality: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/query.c

    // iterate through each database
    for db in handle.syncdbs() {
        // look for a package by name in a database; the database is
        // implemented as a hashmap so this is faster than iterating:
        match db.pkg(package.name()) {
            Ok(pkg) => return Ok(pkg),
            Err(_) => {}
        }
    }
    Err(format!("{:?} not found in the sync databases", package))
}