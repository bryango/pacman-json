//! This module generates different kinds of reverse dependencies from pacman
//! sync databases and gathers them in a big [`HashMap`]. The key ingredient,
//! [`get_reverse_deps_map`], is stolen from <https://github.com/jelly/pacquery>.
//!
//! Note that [`alpm::Package`] does provide reverse dependency information
//! through e.g. [`alpm::Pkg::required_by()`] but it is very slow to enumerate
//! them, possibly due to the mutable data structure [`alpm::AlpmListMut`].
//! Their implementation is hidden in C, but it seems likely that the reverse
//! dependencies are computed on the fly, by enumerating through the whole
//! database for every package when it is called. This is reasonable for a
//! single package query but undesirable if we would like to dump the whole
//! database; thus the reimplementation.
//!
//! See `alpm_sys::ffi::alpm_pkg_compute_requiredby()`.

use alpm::{Alpm, AlpmList, Dep, Package};
use std::collections::{HashMap, HashSet};

/// A map of reverse dependencies, from a package's name to the names of
/// packages that are dependent on it.
pub type RevDepsMap = HashMap<String, HashSet<String>>;

/// Retrieves a HashMap of all reverse dependencies. This function is ported
/// from: <https://github.com/jelly/pacquery>.
pub fn get_reverse_deps_map(
    handle: &Alpm,
    get_dependencies: fn(Package) -> AlpmList<Dep>,
) -> RevDepsMap {
    let mut reverse_deps: RevDepsMap = HashMap::new();
    let dbs = handle.syncdbs();

    for db in dbs {
        for pkg in db.pkgs() {
            for dep in get_dependencies(pkg) {
                reverse_deps
                    .entry(dep.name().to_string())
                    .and_modify(|e| {
                        e.insert(pkg.name().to_string());
                    })
                    .or_insert_with(|| {
                        let mut modify = HashSet::new();
                        modify.insert(pkg.name().to_string());
                        modify
                    });
            }
        }
    }

    reverse_deps
}

/// A collection of all different kinds of reverse dependencies maps.
pub struct ReverseDependencyMaps {
    pub optional_for: RevDepsMap,
    pub required_by: RevDepsMap,
    pub required_by_make: RevDepsMap,
    pub required_by_check: RevDepsMap,
}

/// Generates the full complete reverse dependencies maps from the [`Alpm`]
/// database handle. This is only constructed once after the database is fully
/// initialized.
impl From<&Alpm> for ReverseDependencyMaps {
    fn from(handle: &Alpm) -> Self {
        let get = |f| get_reverse_deps_map(&handle, f);
        Self {
            optional_for: get(|pkg| pkg.optdepends()),
            required_by: get(|pkg| pkg.depends()),
            required_by_make: get(|pkg| pkg.makedepends()),
            required_by_check: get(|pkg| pkg.checkdepends()),
        }
    }
}
