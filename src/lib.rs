#![allow(dead_code)]

pub mod cli;
mod config;
mod dat;
mod dependency;
mod directory;
mod mod_ident;
mod mod_settings;
mod portal;
mod save_file;
mod version;

use crate::cli::{Args, Cmd};
use crate::config::Config;
use crate::dependency::{ModDependency, ModDependencyType};
use crate::directory::Directory;
use crate::mod_ident::ModIdent;
use crate::save_file::SaveFile;
use crate::version::Version;
use anyhow::{anyhow, Result};
use cli::SyncArgs;
use console::style;
use reqwest::blocking::Client;

pub fn run(args: Args) -> Result<()> {
    let config = Config::new(args)?;

    match &config.cmd {
        Cmd::Sync(args) => handle_sync(&config, args),
    }
}

fn handle_sync(config: &Config, args: &SyncArgs) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;

    // Disable mods
    if args.disable_all {
        directory.disable_all();
    }
    for mod_ident in &args.disable {
        directory.disable(mod_ident);
    }

    // Construct download and enable lists
    let mut to_download = vec![];
    let mut to_enable = vec![];
    // Save file
    if let Some(path) = &args.save_file {
        let save_file = SaveFile::from(path.clone())?;

        let mut mods: Vec<ModIdent> = save_file
            .mods
            .iter()
            .filter(|ident| ident.name != "base")
            .cloned()
            .collect();

        if config.sync_latest_versions {
            for mod_ident in mods.iter_mut() {
                mod_ident.version = None;
            }
        }

        to_enable = mods;

        if !args.ignore_startup_settings {
            directory.sync_settings(&save_file.startup_settings)?;
            println!("Synced startup settings");
        }
    }
    // Set
    if let Some(set) = &args.enable_set {
        let sets = config
            .sets
            .as_ref()
            .ok_or_else(|| anyhow!("No mod sets are defined"))?;
        let set = sets
            .get(set)
            .ok_or_else(|| anyhow!("Given set does not exist"))?;
        to_enable = set.to_owned();
    }
    // Enable
    for mod_ident in &args.enable {
        if !to_enable.contains(mod_ident) {
            to_enable.push(mod_ident.clone());
        }
    }
    // Recursively get dependencies to download / enable
    if !args.ignore_deps {
        let mut to_check = to_enable.clone();
        while !to_check.is_empty() {
            let mut to_check_next = vec![];
            for mod_ident in &to_check {
                match get_dependencies(&directory, mod_ident, client) {
                    Ok(dependencies) => {
                        for dependency in dependencies
                            .iter()
                            .filter(|dep| {
                                matches!(
                                    dep.dep_type,
                                    ModDependencyType::Required | ModDependencyType::NoLoadOrder
                                )
                            })
                            .filter(|dep| dep.name != "base")
                        {
                            // TODO: Put this in `Directory`
                            let newest_matching =
                                directory.mods.get(&dependency.name).and_then(|entries| {
                                    match &dependency.version_req {
                                        Some(version_req) => entries.iter().rev().find(|entry| {
                                            version_req.matches(
                                                &entry.ident.get_guaranteed_version().clone(),
                                            )
                                        }),
                                        None => entries.last(),
                                    }
                                });

                            // TODO: Handle if a mod requires a newer version of the dependency
                            if let Some(dep_ident) = newest_matching
                                .map(|dependency| dependency.ident.clone())
                                .or_else(|| {
                                    if !args.no_download {
                                        Some(ModIdent {
                                            name: dependency.name.clone(),
                                            version: None,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .filter(|dep_ident| !to_enable.contains(dep_ident))
                            {
                                to_enable.push(dep_ident.clone());
                                to_check_next.push(dep_ident.clone());
                            }
                        }
                    }
                    // TODO: Prevent download when this occurs
                    Err(err) => eprintln!("{} {}", style("Error downloading mod:").red(), err),
                }
            }
            to_check = to_check_next;
        }
    }

    // TODO: This is bad
    // Add any mods that we don't have to the download list
    for mod_ident in &to_enable {
        if !directory.contains(mod_ident) {
            to_download.push(mod_ident.clone());
        }
    }

    // Download mods
    for mod_ident in &to_download {
        // TODO: Add to to_enable here after download_mod returns a ModIdent
        portal::download_mod(mod_ident, &mut directory, config, client)?;
    }

    // Enable and disable mods
    for mod_ident in &to_enable {
        // TODO: Print errors
        directory.enable(mod_ident)?;
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
        })
        .ok_or_else(|| anyhow!("Failed to retrieve mod dependencies"))
        .or_else(|_| portal::get_dependencies(mod_ident, client))
        .map(|dependencies| {
            dependencies
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
