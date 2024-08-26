pub mod info;
pub mod recurse_deps;
pub mod reverse_deps;
pub mod siglevel;

use alpm::{Alpm, Db, Package, PackageReason};
use clap::Parser;
use info::PackageInfo;
use reverse_deps::ReverseDepsDatabase;

/// Available filters for pacman packages, exposed
/// through the command line interface.
#[derive(Debug, Parser)]
#[command(about)]
pub struct PackageFilters {
    /// Query the sync databases; by default only the local database
    /// (of currently installed packages) is queried
    #[arg(long)]
    pub sync: bool,

    /// Query all packages, including those not explicitly installed;
    /// by default only explicitly installed packages are shown
    #[arg(long)]
    pub all: bool,

    /// Output package info from the current database only; by default we
    /// enrich the output by combining information from both the local
    /// and the sync databases
    #[arg(long)]
    pub plain: bool,

    /// Recursively query the dependencies of the given package;
    /// implies `--all`
    #[arg(long)]
    pub recurse: Option<String>,

    /// `--recurse` installed optional dependencies as well
    #[arg(long, requires = "recurse")]
    pub optional: bool,

    /// `--recurse` dependencies, but only prints package names and versions
    #[arg(long, requires = "recurse")]
    pub summary: bool,
}

impl PackageFilters {
    /// Applies an instance of [`PackageFilters`] to an [`alpm::Package`], and
    /// returns either the desired [`info::PackageInfo`] or an error.
    pub fn generate_pkg_info<'a>(
        &self,
        handle: &'a Alpm,
        pkg: &'a Package,
        reverse_deps: &'a ReverseDepsDatabase,
    ) -> anyhow::Result<PackageInfo<'a>> {
        // only focus on explicitly installed packages
        if self.recurse.is_none() && !self.all && pkg.reason() != PackageReason::Explicit {
            anyhow::bail!("{:?} not explicitly installed, skipped", pkg);
        }
        let mut pkg_info = match self.sync {
            true => PackageInfo::from_sync_pkg(handle, pkg),
            false => PackageInfo::from(pkg),
        };
        if !self.plain {
            pkg_info = self.enrich_pkg_info(handle, pkg_info)
        }
        return Ok(pkg_info.add_reverse_deps(reverse_deps));
    }

    /// Enriches package with sync & local database information, if desired and
    /// when possible. If the sync database information is available and accurate,
    /// it will be preferred as the base info since it contains more details.
    fn enrich_pkg_info<'a>(&self, handle: &'a Alpm, pkg_info: PackageInfo<'a>) -> PackageInfo<'a> {
        if self.sync {
            let sync_info = pkg_info;
            match handle.localdb().pkg(sync_info.name) {
                Err(_) => return sync_info,
                Ok(local_pkg) => {
                    let local_info = PackageInfo::from(local_pkg);
                    return sync_info.add_local_info(local_info);
                }
            };
        }
        // otherwise, the input `pkg` is local:
        let local_info = pkg_info;
        let sync_pkg = match find_in_databases(handle.syncdbs(), local_info.name.to_string()) {
            Err(msg) => {
                eprintln!("{}", msg);
                return local_info;
            }
            Ok(x) => x,
        };
        let sync_info = PackageInfo::from_sync_pkg(handle, sync_pkg);

        return match self.plain
            || local_info.packager != sync_pkg.packager()
            || local_info.version != sync_pkg.version()
        {
            true => local_info.add_sync_info(sync_info),
            false => sync_info.add_local_info(local_info),
        };
    }
}

/// Locates a Package from some databases by its name.
pub fn find_in_databases<'a, T>(databases: T, package: String) -> anyhow::Result<&'a Package>
where
    T: IntoIterator<Item = &'a Db>,
{
    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/packages.rs
    // dump_pkg_search, print_installed: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/package.c
    // display, filter, pkg_get_locality: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/src/pacman/query.c

    // iterate through each database
    for db in databases {
        // look for a package by name in a database; the database is
        // implemented as a hashmap so this is faster than iterating:
        match db.pkg(package.as_str()) {
            Ok(pkg) => return Ok(pkg),
            Err(_) => {}
        }
    }
    anyhow::bail!("{:?} not found in the sync databases", &package)
}
