use pacjump::info::{add_reverse_deps, recurse_dependencies, PackageInfo};
use pacjump::reverse_deps::ReverseDependencyMaps;
use pacjump::siglevel::{default_siglevel, read_conf, repo_siglevel};
use pacjump::{find_in_databases, generate_pkg_info, PackageFilters};

use alpm::Alpm;
use clap::Parser;

/// Dumps json data of the explicitly installed pacman packages.
/// Local packages are matched against the sync databases,
/// and upstream info is added to the output.
fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    unsafe { backtrace_on_stack_overflow::enable() };
    let pkg_filters = PackageFilters::parse();

    let root = read_conf(["RootDir"]).unwrap_or("/".into());
    let db_path = read_conf(["DBPath"]).unwrap_or("/var/lib/pacman/".into());
    let all_repos = read_conf(["--repo-list"]).unwrap_or(["core", "extra", "multilib"].join("\n"));
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

    eprintln!("# generating reverse dependencies ...");
    let reverse_deps = ReverseDependencyMaps::from(&handle);
    eprintln!(
        "# done. Required-by pkgs: {}",
        reverse_deps.required_by.len()
    );
    eprintln!("");

    let db_list = if pkg_filters.sync {
        handle.syncdbs().iter().collect()
    } else {
        vec![handle.localdb()]
    };

    if let Some(name) = pkg_filters.recurse.clone() {
        let pkg = find_in_databases(db_list.clone(), &name)?;
        let pkg_info = generate_pkg_info(&handle, pkg, &pkg_filters)?;
        let info_with_deps = recurse_dependencies(db_list, pkg_info);

        let json = serde_json::to_string(&info_with_deps).expect("failed serializing json");
        println!("{}", json);

        eprintln!("");
        eprintln!("# all done.");

        return Ok(());
    }

    eprintln!("# enumerating packages ...");
    let all_packages: Vec<PackageInfo<'_>> = db_list
        .iter()
        .map(|db| {
            eprintln!("{}: {}", db.name(), db.pkgs().len());
            db.pkgs()
                .iter()
                .filter_map(|pkg| {
                    generate_pkg_info(&handle, pkg, &pkg_filters)
                        .map(|pkg_info| add_reverse_deps(pkg_info, &reverse_deps))
                        .ok()
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect(); // flattened list of packages
    eprintln!("# done. Serializing ...");
    eprintln!("");

    let json = serde_json::to_string(&all_packages).expect("failed serializing json");
    println!("{}", json);

    eprintln!("");
    eprintln!("# all done.");
    Ok(())
}
