use anyhow::Result;

use fmm::input::proc_input;
use fmm::run;

fn main() -> Result<()> {
    let (actions, config, directory) = proc_input()?;

    println!("{:#?}", actions);
    println!("{:#?}", config);

    // run(actions)

    Ok(())
}
