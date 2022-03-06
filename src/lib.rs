#![feature(iter_intersperse)]
#![feature(result_option_inspect)]
#![allow(unused)]

mod cli;
mod config;
mod dat;
mod dependency;
mod directory;
mod mod_settings;
mod portal;
mod save_file;
mod types;

use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use dependency::{ModDependency, ModDependencyType};
use reqwest::blocking::Client;
use semver::Version;
use std::collections::HashSet;
use std::fs;

use crate::cli::{Args, Cmd, SyncCmd};
use crate::config::Config;
use crate::directory::Directory;
use crate::save_file::SaveFile;
use crate::types::*;

pub fn run() -> Result<()> {
    let config = Config::new(Args::parse())?;
    let client = Client::new();

    match &config.cmd {
        Cmd::Sync {
            cmd,
            disable_all,
            ignore_deps,
            no_download,
        } => handle_sync(
            &config,
            &client,
            cmd,
            *disable_all,
            *ignore_deps,
            *no_download,
        ),
    }
}

fn handle_sync(
    config: &Config,
    client: &Client,
    cmd: &SyncCmd,
    disable_all: bool,
    ignore_deps: bool,
    no_download: bool,
) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;

    if disable_all {
        directory.disable_all();
    }

    let mut to_download = vec![];
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
            let set = sets
                .get(set_name)
                .ok_or_else(|| anyhow!("Given set does not exist"))?;
            to_enable = set.to_owned();
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
                    mod_ident.version = None;
                }
            }

            to_enable = mods;

            if !ignore_startup_settings {
                directory.sync_settings(&save_file.startup_settings)?;
                println!("Synced startup settings");
            }
        }
    }

    // Recursively extract dependencies and add them to the download/enable lists
    if !ignore_deps {
        let mut to_check = to_enable.clone();
        while !to_check.is_empty() {
            let mut to_check_next = vec![];
            for mod_ident in &to_check {
                for dependency in get_dependencies(&directory, mod_ident, client)?
                    .iter()
                    .filter(|dep| {
                        matches!(
                            dep.dep_type,
                            ModDependencyType::Required | ModDependencyType::NoLoadOrder
                        )
                    })
                    .filter(|dep| dep.name != "base")
                {
                    let newest_matching =
                        directory
                            .mods
                            .get(&dependency.name)
                            .and_then(|entries| match &dependency.version_req {
                                Some(version_req) => entries.iter().rev().find(|entry| {
                                    version_req
                                        .matches(&entry.ident.get_guaranteed_version().clone())
                                }),
                                None => entries.last(),
                            });

                    // TODO: Handle if a mod requires a newer version of the dependency
                    let dep_ident = if let Some(dependency) = newest_matching {
                        dependency.ident.clone()
                    } else {
                        ModIdent {
                            name: dependency.name.clone(),
                            version: None,
                        }
                    };

                    if !to_enable.contains(&dep_ident) {
                        to_enable.push(dep_ident.clone());
                        to_check_next.push(dep_ident.clone());
                    }
                }
            }
            to_check = to_check_next;
        }
    }

    // Add any mods that we don't have to the download list
    for mod_ident in &to_enable {
        if !directory.contains(mod_ident) {
            to_download.push(mod_ident.clone());
        }
    }

    // Download mods
    for mod_ident in to_download {
        // TODO: Add to to_enable here after download_mod returns a ModIdent
        portal::download_mod(&mod_ident, &mut directory, config, client)?;
    }

    // Enable and disable mods
    for mod_ident in to_enable {
        // TODO: Print errors
        directory.enable(&mod_ident)?;
    }
    for mod_ident in to_disable {
        directory.disable(&mod_ident);
    }

    // Write mod-list.json
    directory.save()?;

    Ok(())
}

fn get_dependencies(
    directory: &Directory,
    mod_ident: &ModIdent,
    client: &Client,
) -> Result<Vec<ModDependency>> {
    directory
        .mods
        .get(&mod_ident.name)
        .and_then(|mod_entries| crate::get_mod_version(mod_entries, mod_ident))
        .map(|mod_entry| {
            directory::read_info_json(&mod_entry.entry)
                .and_then(|info_json| info_json.dependencies)
                .unwrap_or_default()
                .iter()
                .filter(|dependency| {
                    dependency.name != "base"
                        && matches!(
                            dependency.dep_type,
                            ModDependencyType::NoLoadOrder | ModDependencyType::Required
                        )
                })
                .cloned()
                .collect()
        })
        .ok_or_else(|| anyhow!("Failed to retrieve mod dependencies"))
        .or_else(|_| portal::get_dependencies(mod_ident, client))
}

trait HasVersion {
    fn get_version(&self) -> &Version;
}

fn get_mod_version<'a, T: HasVersion>(list: &'a [T], mod_ident: &ModIdent) -> Option<&'a T> {
    if let Some(version) = &mod_ident.version {
        list.iter()
            .rev()
            .find(|entry| version == entry.get_version())
    } else {
        list.last()
    }
}
