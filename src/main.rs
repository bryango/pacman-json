use pacjump::info::PackageInfo;
use pacjump::reverse_deps::ReverseDepsDatabase;
use pacjump::{alpm_default, find_in_databases, get_databases, PackageFilters};

use clap::Parser;
use indexmap::IndexSet;

/// Dumps json data of the explicitly installed pacman packages.
/// Local packages are matched against the sync databases,
/// and upstream info is added to the output.
fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    #[cfg(feature = "backtrace-overflow")]
    unsafe {
        backtrace_on_stack_overflow::enable()
    };
    let pkg_filters = PackageFilters::parse();
    let handle = &alpm_default()?;

    let reverse_deps = match pkg_filters.no_reverse || pkg_filters.summary {
        true => {
            eprintln!("# skip generating reverse dependencies ...");
            ReverseDepsDatabase::default()
        }
        false => {
            eprintln!("# generating reverse dependencies ...");
            ReverseDepsDatabase::populate(handle, pkg_filters.find_providers)
        }
    };
    eprintln!(
        "# done. Required-by pkgs: {}",
        reverse_deps.required_by.len()
    );
    eprintln!("");

    let databases = get_databases(handle, pkg_filters.sync);
    let all_packages: Vec<PackageInfo<'_>> = if let Some(name) = &pkg_filters.recurse {
        let pkg = find_in_databases(databases.clone(), name)?;
        let pkg_info = pkg_filters.generate_pkg_info(handle, pkg, &reverse_deps)?;
        let mut deps_set = IndexSet::new();
        let mut deps_pkgs = Vec::new();
        let _ = pkg_info.recurse_dependencies(
            handle,
            databases,
            &pkg_filters,
            &reverse_deps,
            0,
            &mut deps_set,
            &mut deps_pkgs,
        );

        eprintln!("");
        eprintln!("{:#?}", deps_set);
        eprintln!("# '{}' closure: {} packages", name, deps_set.len());
        eprintln!("");

        if pkg_filters.summary {
            for dep in deps_set {
                println!("{}", dep)
            }
            return Ok(());
        }

        deps_pkgs.reverse();
        deps_pkgs
    } else {
        eprintln!("# enumerating all packages ...");
        databases
            .iter()
            .map(|db| {
                eprintln!("{}: {}", db.name(), db.pkgs().len());
                db.pkgs()
                    .iter()
                    .filter_map(|pkg| {
                        pkg_filters
                            .generate_pkg_info(handle, pkg, &reverse_deps)
                            .ok()
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect() // flattened list of packages
    };

    eprintln!("# done. Serializing ...");
    eprintln!("");

    let json = serde_json::to_string(&all_packages).expect("failed serializing json");
    println!("{}", json);

    eprintln!("");
    eprintln!("# all done.");
    Ok(())
}
