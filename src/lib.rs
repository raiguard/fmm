#![feature(iter_intersperse)]
#![allow(unused)]

mod cli;
mod config;
mod dependency;
mod directory;
mod download;
mod mod_settings;
mod read;
mod save_file;
mod types;

use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use reqwest::blocking::Client;
use semver::Version;
use std::fs;

use crate::cli::{Args, Cmd, SyncCmd};
use crate::config::Config;
use crate::directory::Directory;
use crate::save_file::SaveFile;
use crate::types::*;

pub fn run() -> Result<()> {
    let config = Config::new(Args::parse())?;
    // println!("{:#?}", config);

    // let client = Client::new();

    match &config.cmd {
        Cmd::Sync {
            cmd,
            disable_all,
            ignore_deps,
            no_download,
        } => handle_sync(&config, cmd, *disable_all, *ignore_deps, *no_download),
    }
}

fn handle_sync(
    config: &Config,
    cmd: &SyncCmd,
    disable_all: bool,
    ignore_deps: bool,
    no_download: bool,
) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;

    if disable_all {
        directory.disable_all();
    }

    let mut to_enable = vec![];
    let mut to_disable = vec![];

    // Get initial lists
    match cmd {
        SyncCmd::Enable { mods } => to_enable = mods.clone(),
        SyncCmd::EnableSet { set } => {
            let set_name = set
                .as_ref()
                .ok_or_else(|| anyhow!("Did not provide a set name"))?;
            let sets = config
                .sets
                .as_ref()
                .ok_or_else(|| anyhow!("No mod sets are defined"))?;
            if let Some(set) = sets.get(set_name) {
                to_enable = set.to_owned();
            }
        }
        SyncCmd::Disable { mods } => to_disable = mods.clone(),
        SyncCmd::SaveFile {
            path,
            ignore_startup_settings,
        } => {
            let save_file = SaveFile::from(path.clone())?;

            let mut mods = save_file.mods.to_vec();

            if config.sync_latest_versions {
                for mod_ident in mods.iter_mut() {
                    mod_ident.version_req = None;
                }
            }

            to_enable = mods;

            if !ignore_startup_settings {
                directory.sync_settings(&save_file.startup_settings)?;
                println!("Synced startup settings");
            }
        }
        SyncCmd::Upgrade { mods } => todo!(),
    }

    // Extract dependencies and add them to the list
    if !ignore_deps {
        // TODO:
    }

    // Enable and disable mods
    for mod_ident in to_enable {
        // TODO: Print errors
        directory.enable(&mod_ident)?;
    }
    for mod_ident in to_disable {
        directory.disable(&mod_ident);
    }

    // TODO: Make this a Directory method
    // Write mod-list.json
    fs::write(
        &directory.mod_list_path,
        serde_json::to_string_pretty(&ModListJson {
            mods: directory.mod_list,
        })?,
    )?;

    Ok(())
}

// // Enable and/or download specified mods
// let mut cycle_orders: Vec<ManageOrder> = app
//     .enable
//     .iter()
//     .map(|mod_ident| ManageOrder::Enable(mod_ident.clone()))
//     .collect();

// while !cycle_orders.is_empty() {
//     cycle_orders = cycle_orders
//         .iter_mut()
//         .filter(|order| order.get_name() != "base")
//         .filter_map(|order| match order {
//             ManageOrder::Download(mod_ident) => {
//                 if config.auto_download {
//                     if download::download_mod(
//                         mod_ident,
//                         &mut directory,
//                         &config,
//                         &client,
//                     )
//                     .ok()?
//                     {
//                         Some(vec![ManageOrder::Enable(mod_ident.clone())])
//                     } else {
//                         None
//                     }
//                 } else {
//                     println!("{} {}", style("Did not download").red(), mod_ident.name);
//                     None
//                 }
//             }
//             ManageOrder::Enable(mod_ident) => directory.enable(mod_ident),
//         })
//         .flatten()
//         .collect();
// }

trait HasVersion {
    fn get_version(&self) -> &Version;
}

fn get_mod_version<'a, T: HasVersion>(list: &'a [T], mod_ident: &ModIdent) -> Option<&'a T> {
    if let Some(version_req) = &mod_ident.version_req {
        list.iter()
            .rev()
            .find(|entry| version_req.matches(entry.get_version()))
    } else {
        list.last()
    }
}
