use pacjump::info::PackageInfo;
use pacjump::recurse_deps::recurse_dependencies;
use pacjump::reverse_deps::ReverseDepsDatabase;
use pacjump::siglevel::{default_siglevel, repo_siglevel};
use pacjump::{find_in_databases, get_databases, read_conf, PackageFilters};

use alpm::Alpm;
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

    let root = read_conf(["RootDir"]).unwrap_or("/".into());
    let db_path = read_conf(["DBPath"]).unwrap_or("/var/lib/pacman/".into());
    let all_repos = read_conf(["--repo-list"]).unwrap_or(["core", "extra", "multilib"].join("\n"));
    eprintln!("RootDir: {root}");
    eprintln!("DBPath: {db_path}");

    let default_siglevel = default_siglevel();
    eprintln!("SigLevel::{default_siglevel:?}");
    eprintln!("");

    let handle = &Alpm::new(root, db_path).unwrap();

    // register sync databases from pacman.conf
    eprintln!("--repo-list:");
    for repo in all_repos.split_terminator('\n') {
        let sig_level = repo_siglevel(repo, default_siglevel);
        handle.register_syncdb(repo, sig_level).unwrap();
        eprintln!("{repo}: SigLevel::{sig_level:?}");
    }
    eprintln!("");

    eprintln!("# generating reverse dependencies ...");
    let reverse_deps = ReverseDepsDatabase::from(handle);
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
        let _ = recurse_dependencies(
            handle,
            databases,
            &pkg_filters,
            &reverse_deps,
            pkg_info,
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
