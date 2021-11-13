#![feature(iter_intersperse)]

use anyhow::anyhow;
use anyhow::Result;
use std::fs;

mod dependency;
mod directory;
pub mod input;
mod sync;
mod types;

use directory::Directory;
use input::Actions;
use types::ModListJson;

pub fn run(mut actions: Actions) -> Result<()> {
    // let mut directory = Directory::new(match actions.mods_dir {
    //     Some(dir) => dir,
    //     None => {
    //         return Err(anyhow!(
    //             "Must specify a directory path via flag or via the configuration file."
    //         ))
    //     }
    // })?;

    // // List mods
    // if app.list {
    //     let mut lines: Vec<String> = directory
    //         .mods
    //         .iter()
    //         .flat_map(|(mod_name, mod_versions)| {
    //             mod_versions
    //                 .iter()
    //                 .map(|version| format!("{} v{}", mod_name, version.version))
    //                 .collect::<Vec<String>>()
    //         })
    //         .collect();

    //     lines.sort();
    //     lines.iter().for_each(|line| println!("{}", line));
    // }

    // // Sync with save
    // if let Some(sync_path) = app.sync {
    //     let save_file = sync::SaveFile::from(sync_path)?;

    //     app.enable = save_file.mods;
    // }

    // // Remove specified mods
    // for mod_ident in app.remove {
    //     if mod_ident.name != "base" {
    //         directory.remove(&mod_ident);
    //     }
    // }

    // // Disable all mods
    // if app.disable_all {
    //     directory.disable_all();
    // }

    // // Disable specified mods
    // for mod_ident in app.disable {
    //     directory.disable(&mod_ident);
    // }

    // // Enable all mods
    // if app.enable_all {
    //     directory.enable_all();
    // }

    // // Enable specified mods
    // let mut to_enable = app.enable;
    // while !to_enable.is_empty() {
    //     to_enable = to_enable
    //         .iter_mut()
    //         .filter(|mod_ident| mod_ident.name != "base")
    //         .filter_map(|mod_ident| directory.enable(mod_ident))
    //         .flatten()
    //         .collect();
    // }

    // // Write mod-list.json
    // fs::write(
    //     &directory.mod_list_path,
    //     serde_json::to_string_pretty(&ModListJson {
    //         mods: directory.mod_list,
    //     })?,
    // )?;

    Ok(())
}
