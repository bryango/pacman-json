//! This produces a binary that generates completions for `pacjump` at the
//! current working directory.
//!
//! This is also stolen from <https://github.com/jelly/pacquery>.

use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use std::env;

use pacjump::PackageFilters;

fn main() -> anyhow::Result<()> {
    let out_dir = env::current_dir()?;
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
