use alpm::Alpm;
use std::collections::{HashMap, HashSet};

/// Retrieves a HashMap of all reverse dependencies. This function is ported
/// from: https://github.com/jelly/pacquery.
fn get_reverse_deps_map(pacman: &Alpm) -> HashMap<String, HashSet<String>> {
    let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();
    let dbs = pacman.syncdbs();

    for db in dbs {
        for pkg in db.pkgs() {
            for dep in pkg.depends() {
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

            for dep in pkg.makedepends() {
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

            for dep in pkg.checkdepends() {
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
