use anyhow::Result;
use clap::Parser;
use fmm::cli::Args;
use fmm::run;

fn main() -> Result<()> {
    run(Args::parse())
}
