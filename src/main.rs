use anyhow::Result;
use clap::Parser;
use fmm::cli::Args;
use fmm::run;

// TODO: Do our own error handling?
fn main() -> Result<()> {
    run(Args::parse())
}
