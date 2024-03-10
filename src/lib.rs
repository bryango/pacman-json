use clap::Parser;

#[derive(Debug, Parser)]
#[command(about)]
pub struct PackageFilters {
    /// Query the sync databases; by default we only query the local database
    /// with the currently installed packages.
    #[arg(long)]
    pub sync: bool,

    /// Query all packages, including those not explicitly installed;
    /// by default only explicitly installed packages are shown.
    #[arg(long)]
    pub all: bool,

    /// Output package info from the current database only; by default we
    /// enrich the output by combining information from both the local
    /// and the sync databases.
    #[arg(long)]
    pub plain: bool,
}
