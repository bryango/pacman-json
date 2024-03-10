//! This produces a binary that generates completions for `pacman-json` at the
//! current working directory.
//!
//! This is also stolen from <https://github.com/jelly/pacquery>.

use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use std::env;

use pacman_json::PackageFilters;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var_os("PWD").expect("$PWD should have been set");
    for variant in Shell::value_variants() {
        clap_complete::generate_to(
            *variant,
            &mut PackageFilters::command(),
            env!("CARGO_PKG_NAME"),
            &out_dir,
        )?;
    }
    println!("# completion scripts generated in {:?}", out_dir);
    Ok(())
}
