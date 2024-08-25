//! A module that generates different kinds of reverse dependencies from pacman
//! sync databases and gathers them in a big [HashMap] from package names to
//! their respective [BTreeSet]s of reverse dependencies. The key ingredient,
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
//! See: [`alpm_sys::alpm_pkg_compute_requiredby()`]
//!

use alpm::{Alpm, AlpmList, Dep, Package};
use std::collections::{BTreeSet, HashMap};

/// A wrapper of [`BTreeSet`] for reverse dependencies.
#[derive(Debug, derive_more::Deref, derive_more::DerefMut, serde::Serialize)]
pub struct ReverseDeps(BTreeSet<String>);
impl ReverseDeps {
    /// A constant reference to an empty set of reverse dependencies.
    pub const NONE: &'static Self = &Self::new();

    /// Makes a new, empty set of reverse dependencies.
    pub const fn new() -> Self {
        ReverseDeps(BTreeSet::new())
    }
}
impl Default for &ReverseDeps {
    fn default() -> Self {
        ReverseDeps::NONE
    }
}

/// A map of reverse dependencies, from a package's name to the names of
/// packages that are dependent on it.
pub type ReverseDepsMap = HashMap<String, ReverseDeps>;

/// Retrieves a HashMap of all reverse dependencies. This function is ported
/// from: <https://github.com/jelly/pacquery>.
pub fn get_reverse_deps_map(
    handle: &Alpm,
    get_dependencies: fn(&Package) -> AlpmList<&Dep>,
) -> ReverseDepsMap {
    let mut reverse_deps: ReverseDepsMap = HashMap::new();
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
                        let mut modify = ReverseDeps::new();
                        modify.insert(pkg.name().to_string());
                        modify
                    });
            }
        }
    }

    reverse_deps
}

/// A collection of all different kinds of reverse dependencies maps.
pub struct ReverseDepsDatabase {
    pub optional_for: ReverseDepsMap,
    pub required_by: ReverseDepsMap,
    pub required_by_make: ReverseDepsMap,
    pub required_by_check: ReverseDepsMap,
}

impl From<&Alpm> for ReverseDepsDatabase {
    /// Generates the full complete reverse dependencies maps from the [`Alpm`]
    /// database handle. This is only constructed once after the database is
    /// fully initialized.
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
