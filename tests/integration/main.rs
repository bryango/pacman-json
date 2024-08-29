use duct::cmd;

#[test]
fn explicits() {
    let ours = cmd!("cargo", "run", "--")
        .pipe(cmd!("jq", "length"))
        .read()
        .unwrap();
    let refs = cmd!("pacman", "-Qe").pipe(cmd!("wc", "-l")).read().unwrap();
    debug_assert_eq!(ours, refs)
}

#[test]
fn all_packages() {
    let all = cmd!("cargo", "run", "--", "--all", "--no-reverse");
    let ours = all.pipe(cmd!("jq", "length")).read().unwrap();
    let refs = cmd!("pacman", "-Q").pipe(cmd!("wc", "-l")).read().unwrap();
    debug_assert_eq!(ours, refs)
}

#[test]
fn recurse() {
    let pkg = "bash";
    let sed = cmd!("sed", "-E", "s/^(.*[^><=])[><=].*/\\1/");
    let ours = cmd!(
        "cargo",
        "run",
        "--",
        "--recurse",
        pkg,
        "--summary",
        "--no-reverse"
    )
    .pipe(sed.clone())
    .read()
    .unwrap();
    let refs = cmd!("pactree", pkg, "--unique")
        .pipe(sed.clone())
        .read()
        .unwrap();
    debug_assert_eq!(ours, refs)
}
