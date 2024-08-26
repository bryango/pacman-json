use alpm::{Alpm, AlpmListMut, Db};
use indexmap::IndexSet;

use crate::info::{DepInfo, PacList, PackageInfo};
use crate::reverse_deps::ReverseDepsDatabase;
use crate::PackageFilters;

/// Recurses the dependency tree of a [`PackageInfo`], finds the packages
/// satisfying the dependency requirements, collects the satisfiers' data
/// into a mutable [`IndexSet`], and adds the satisfiers' [`PackageInfo`]s
/// into a mutable [`Vec`].
pub fn recurse_dependencies<'h, T>(
    handle: &'h Alpm,
    databases: T,
    pkg_filters: &PackageFilters,
    reverse_deps: &'h ReverseDepsDatabase,
    pkg_info: PackageInfo<'h>,
    depth: u64,
    deps_set: &mut IndexSet<String>,
    deps_pkgs: &mut Vec<PackageInfo<'h>>,
) -> ()
where
    T: IntoIterator<Item = &'h Db> + Clone,
{
    eprintln!(
        "# level {}: recursing into '{}': {:?}\n",
        depth, pkg_info.name, pkg_info.depends_on
    );
    deps_set.insert(format!("{}={}", pkg_info.name, pkg_info.version));
    let mut_list = AlpmListMut::from_iter(databases.clone().into_iter());
    let db_list = mut_list.list();
    let mut satisfied_dependencies = |dependencies: PacList<DepInfo<'h>>| -> Vec<DepInfo<'h>> {
        dependencies
            .into_iter()
            .map(|dep| {
                let pkg = match db_list.clone().find_satisfier(dep.dep_string.clone()) {
                    Some(pkg) => pkg,
                    None => return dep.clone(),
                };
                let satisfier = format!("{}={}", pkg.name(), pkg.version());
                let next_depth = depth + 1;
                if !deps_set.contains(&satisfier) {
                    let pkg_info = pkg_filters
                        .generate_pkg_info(handle, pkg, &reverse_deps)
                        .unwrap_or(pkg.into());
                    recurse_dependencies(
                        &handle,
                        databases.clone(),
                        pkg_filters,
                        &reverse_deps,
                        pkg_info,
                        next_depth,
                        deps_set,
                        deps_pkgs,
                    );
                } else {
                    eprintln!(
                        "# level {}: duplicated dependency: '{}' provides '{}'",
                        next_depth,
                        satisfier,
                        dep.dep_string.clone()
                    );
                }
                DepInfo {
                    satisfier: Some(satisfier),
                    ..dep.clone()
                }
            })
            .collect()
    };
    let pkg_info = match pkg_filters.optional {
        true => PackageInfo {
            depends_on: satisfied_dependencies(pkg_info.depends_on).into(),
            optional_deps: satisfied_dependencies(pkg_info.optional_deps).into(),
            ..pkg_info
        },
        false => PackageInfo {
            depends_on: satisfied_dependencies(pkg_info.depends_on).into(),
            ..pkg_info
        },
    };
    if pkg_filters.summary {
        return;
    } else {
        deps_pkgs.push(pkg_info);
    }
}
