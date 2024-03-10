use alpm::{Alpm, AlpmList, Dep, Package};
use std::collections::{HashMap, HashSet};

pub type RevDepsMap = HashMap<String, HashSet<String>>;

/// Retrieves a HashMap of all reverse dependencies. This function is ported
/// from: https://github.com/jelly/pacquery.
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

pub struct ReverseDependencyMaps {
    pub optional_for: RevDepsMap,
    pub required_by: RevDepsMap,
    pub required_by_make: RevDepsMap,
    pub required_by_check: RevDepsMap,
}

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
