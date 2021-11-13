use anyhow::Result;
use structopt::StructOpt;

use fmm::cli;
use fmm::run;

fn main() -> Result<()> {
    let app = cli::App::from_args();
    run(app)
}
