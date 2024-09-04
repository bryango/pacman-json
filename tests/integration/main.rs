//! Integration tests for `pacjump`.
//! Requires:
//! - `pacman-contrib` for `pactree`
//! - `jq`
//! Implicits:
//! - `cargo`
//! - `base`
//!   - `glibc`
//!   - `pacman`
//!   - `coreutils` for `wc`
//!   - `bash`
//!   - `sed`

use std::sync::Once;

fn split_cli_args(args: &str) -> Box<[&str]> {
    args.split_whitespace().collect()
}

fn cmd<T>(args: T) -> duct::Expression
where
    T: AsRef<str>,
{
    fn inner(args: &str) -> duct::Expression {
        let all_args = split_cli_args(args);
        let mut args = all_args.iter();
        let program = *args.next().unwrap();
        duct::cmd(program, args)
    }
    inner(args.as_ref())
}

fn cargo_run<T>(args: T) -> duct::Expression
where
    T: AsRef<str>,
{
    fn inner(args: &str) -> duct::Expression {
        let flags = match cfg!(tarpaulin) {
            true => "--target-dir=target/tarpaulin",
            false => "",
        };
        let args = format!("cargo run {flags} -- {args}");
        cmd(args.trim())
    }
    inner(args.as_ref())
}

static INIT: Once = Once::new();

fn dump_all() -> String {
    let mut all = String::new();
    INIT.call_once(|| {
        all = cargo_run("--all --no-reverse").read().unwrap();
    });
    all
}

#[test]
fn explicits_with_reverse_providers() {
    let ours = cargo_run("--find-providers").pipe(cmd("jq length")).read().unwrap();
    let refs = cmd("pacman -Qe").pipe(cmd("wc -l")).read().unwrap();
    debug_assert_eq!(ours, refs)
}

#[test]
fn all_packages_without_reverse_deps() {
    let all = dump_all();
    let ours = cmd("jq length").stdin_bytes(all).read().unwrap();
    let refs = cmd("pacman -Q").pipe(cmd("wc -l")).read().unwrap();
    debug_assert_eq!(ours, refs);
}

#[test]
fn recurse_not_found() {
    let pkg = "imaginary-fake-package";
    cargo_run(format!("--recurse {pkg}")).run().unwrap_err();
}

#[test]
fn recurse_json() {
    let pkg = "bash";
    let ours = cargo_run(format!("--recurse {pkg}"))
        .pipe(cmd("jq length"))
        .read()
        .unwrap();
    let refs = cmd(format!("pactree --unique {pkg}"))
        .pipe(cmd("wc -l"))
        .read()
        .unwrap();
    debug_assert_eq!(ours, refs)
}

#[test]
fn recurse_optional_deps() {
    let pkg = "glibc";
    cargo_run(format!("--summary --recurse {pkg} --optional"))
        .run()
        .unwrap();
}

#[test]
fn recurse_summary() {
    let pkg = "bash";
    let sed = cmd("sed -E s/^(.*[^><=])[><=].*/\\1/");
    let ours = cargo_run(format!("--summary --recurse {pkg}"))
        .pipe(sed.clone())
        .read()
        .unwrap();
    let refs = cmd(format!("pactree --unique {pkg}"))
        .pipe(sed.clone())
        .read()
        .unwrap();
    debug_assert_eq!(ours, refs)
}
