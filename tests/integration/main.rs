use duct::cmd;

#[test]
fn explicits() {
    let ours = cmd!("cargo", "run", "--")
        .pipe(cmd!("jq", "length"))
        .read()
        .unwrap();
    let refs = cmd!("pacman", "-Qe").pipe(cmd!("wc", "-l")).read().unwrap();
    assert_eq!(ours, refs)
}

#[test]
fn all_packages() {
    let all = cmd!("cargo", "run", "--", "--all", "--no-reverse");
    let ours = all.pipe(cmd!("jq", "length")).read().unwrap();
    let refs = cmd!("pacman", "-Q").pipe(cmd!("wc", "-l")).read().unwrap();
    assert_eq!(ours, refs)
}
