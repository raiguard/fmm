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

use crate::cli::{Args, Cmd, SyncArgs};
use crate::config::Config;
use crate::dependency::{ModDependency, ModDependencyType};
use crate::directory::Directory;
use crate::mod_ident::ModIdent;
use crate::portal::Portal;
use crate::save_file::SaveFile;
use crate::version::Version;
use anyhow::{anyhow, Result};

pub fn run(args: Args) -> Result<()> {
    let config = Config::new(args)?;

    match &config.cmd {
        Cmd::Sync(args) => handle_sync(&config, args),
    }
}

fn handle_sync(config: &Config, args: &SyncArgs) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;
    let mut portal = Portal::new();

    // Disable mods
    if args.disable_all {
        directory.disable_all();
    }
    for ident in &args.disable {
        directory.disable(ident);
    }

    // Construct initial enable list
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
            for ident in mods.iter_mut() {
                ident.version = None;
            }
        }

        if !args.ignore_startup_settings {
            directory
                .settings
                .merge_startup_settings(&save_file.startup_settings)?;
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
    for ident in &args.enable {
        if !to_enable.contains(ident) {
            to_enable.push(ident.clone());
        }
    }

    // Split into to_download and to_enable lists
    let (mut to_download, mut to_enable): (Vec<_>, Vec<_>) = to_enable
        .iter()
        .cloned()
        .partition(|ident| directory.contains(ident));

    // Recursively get dependencies to download / enable
    if !args.ignore_deps {
        let mut to_check = to_enable.clone();
        while !to_check.is_empty() {
            let mut to_check_next = vec![];
            for ident in &to_check {
                let dependencies = if let Some(dir_mod) = directory.get(ident) {
                    dir_mod
                        .get_release(ident)
                        .and_then(|release| release.get_dependencies())
                } else {
                    if !portal.contains(&ident.name) {
                        portal.fetch(&ident.name)?;
                    }
                    portal
                        .get(&ident.name)
                        .and_then(|mod_data| mod_data.get_release(ident))
                        .and_then(|release| release.get_dependencies())
                };

                if let Some(dependencies) = dependencies {
                    for dependency in dependencies.iter().filter(|dependency| {
                        matches!(
                            dependency.dep_type,
                            ModDependencyType::Required | ModDependencyType::NoLoadOrder
                        )
                    }) {
                        // TODO: Create a trait that we can share between directory and portal for this part
                        if let Some(release) =
                            directory.get_newest_matching(&ident.name, &dependency.version_req)
                        {
                            to_enable.push(release.ident.clone());
                        } else if let Some(release) =
                            portal.get_newest_matching(&ident.name, &dependency.version_req)
                        {
                            to_download.push(ModIdent {
                                name: dependency.name.clone(),
                                version: Some(release.get_version().clone()),
                            });
                        }
                    }
                }
            }
            to_check = to_check_next;
        }
    }

    // Download mods
    for ident in &to_download {
        let (new_ident, path) = portal.download(ident, config)?;
        directory.add(
            new_ident.clone(),
            std::fs::read_dir(&config.mods_dir)?
                .filter_map(|entry| entry.ok())
                .find(|entry| entry.path() == path)
                .unwrap(),
        );
        to_enable.push(new_ident);
    }

    // Enable and disable mods
    for ident in &to_enable {
        // TODO: Print errors
        directory.enable(ident)?;
    }

    // Write mod-list.json
    directory.save()?;

    Ok(())
}

trait HasReleases<T: HasVersion> {
    fn get_release(&self, ident: &ModIdent) -> Option<&T> {
        if let Some(version) = &ident.version {
            self.get_release_list()
                .iter()
                .rev()
                .find(|entry| version == entry.get_version())
        } else {
            self.get_release_list().last()
        }
    }

    fn get_release_list(&self) -> &[T];
}

trait HasVersion {
    fn get_version(&self) -> &Version;
}

trait HasDependencies {
    fn get_dependencies(&self) -> Option<&Vec<ModDependency>>;
}
