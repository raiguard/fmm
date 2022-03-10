#![allow(unstable_name_collisions)]

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

use crate::cli::{Args, Cmd, QueryArgs, SyncArgs};
use crate::config::Config;
use crate::dependency::{ModDependency, ModDependencyType};
use crate::directory::Directory;
use crate::mod_ident::ModIdent;
use crate::portal::Portal;
use crate::save_file::SaveFile;
use crate::version::Version;
use anyhow::{anyhow, Context, Result};
use console::style;
use itertools::Itertools;

pub fn run(args: Args) -> Result<()> {
    let config = Config::new(args)?;

    match &config.cmd {
        Cmd::Sync(args) => handle_sync(&config, args),
        Cmd::Query(args) => handle_query(&config, args),
    }
}

fn handle_sync(config: &Config, args: &SyncArgs) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir).context("Could not build mod registry")?;
    let mut portal = Portal::new();

    // Remove mods
    for ident in &args.remove {
        if let Err(e) = directory.remove(ident) {
            eprintln!("{} {}", style("Error:").red().bold(), e);
        }
    }

    // Disable mods
    if args.disable_all {
        directory.disable_all();
    }
    for ident in &args.disable {
        directory.disable(ident);
    }

    // Construct initial enable list
    let mut to_enable_input = vec![];
    // Save file
    if let Some(path) = &args.save_file {
        let save_file =
            SaveFile::from(path.clone()).context(format!("Could not sync with {:?}", path))?;

        let mut mods: Vec<ModIdent> = save_file
            .mods
            .iter()
            .filter(|ident| ident.name != "base")
            .cloned()
            .collect();

        for ident in mods.iter_mut() {
            if config.sync_latest_versions {
                ident.version = None;
            }
            to_enable_input.push(ident.clone());
        }

        if !args.ignore_startup_settings {
            directory
                .settings
                .merge_startup_settings(&save_file.startup_settings)
                .context("Could not sync startup settings")?;
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
        to_enable_input = set.to_owned();
    }
    // Enable
    for ident in &args.enable {
        if !to_enable_input.contains(ident) {
            to_enable_input.push(ident.clone());
        }
    }

    // Iterate mods and dependencies to determine what to download and enable
    let mut to_download = vec![];
    let mut to_enable = vec![];
    if !args.ignore_deps {
        let mut to_check = to_enable_input;
        while !to_check.is_empty() {
            let mut to_check_next = vec![];
            for ident in &to_check {
                let dependencies = if let Some(dir_mod) = directory.get(ident) {
                    dir_mod.get_release(ident).and_then(|release| {
                        if !to_enable.contains(&release.ident) {
                            to_enable.push(release.ident.clone());
                            release.get_dependencies()
                        } else {
                            None
                        }
                    })
                } else {
                    if !portal.contains(&ident.name) {
                        portal.fetch(&ident.name);
                    }
                    portal
                        .get(&ident.name)
                        .and_then(|mod_data| mod_data.get_release(ident))
                        .and_then(|release| {
                            if !to_download.contains(ident) {
                                to_download.push(ModIdent {
                                    name: ident.name.clone(),
                                    version: Some(release.get_version().clone()),
                                });
                                release.get_dependencies()
                            } else {
                                None
                            }
                        })
                };

                if let Some(dependencies) = dependencies {
                    // FIXME: Don't clone here if possible
                    for dependency in dependencies.clone().iter().filter(|dependency| {
                        dependency.name != "base"
                            && matches!(
                                dependency.dep_type,
                                ModDependencyType::Required | ModDependencyType::NoLoadOrder
                            )
                    }) {
                        if let Some(dep_release) = directory.get_newest_matching(dependency) {
                            to_check_next.push(dep_release.ident.clone());
                        } else {
                            if !portal.contains(&dependency.name) {
                                portal.fetch(&dependency.name);
                            }
                            if let Some(dep_release) = portal.get_newest_matching(dependency) {
                                to_check_next.push(ModIdent {
                                    name: dependency.name.clone(),
                                    version: Some(dep_release.get_version().clone()),
                                });
                            }
                        }
                    }
                }
            }
            to_check = to_check_next;
        }
    }

    // Download mods
    for ident in &to_download {
        match portal.download(ident, config) {
            Ok((new_ident, path)) => {
                directory.add(new_ident.clone(), path);
                to_enable.push(new_ident);
            }
            Err(e) => eprintln!("{} {}", style("Error").red().bold(), e),
        };
    }

    // Enable mods
    for ident in &to_enable {
        if let Err(e) = directory.enable(ident) {
            eprintln!("{} {}", style("Error").red().bold(), e);
        }
    }

    // Write mod-list.json
    directory.save()?;

    Ok(())
}

fn handle_query(config: &Config, args: &QueryArgs) -> Result<()> {
    let portal = Portal::new();

    let mod_list = portal.get_all_mods()?;

    let query = args.query.to_lowercase();

    let results = mod_list
        .results
        .into_iter()
        .filter(|mod_data| mod_data.latest_release.is_some())
        .filter(|mod_data| {
            mod_data.name.to_lowercase().contains(&query)
                || mod_data.owner.to_lowercase().contains(&query)
                || mod_data.title.to_lowercase().contains(&query)
                || mod_data.summary.to_lowercase().contains(&query)
        })
        .sorted_by(|m1, m2| Ord::cmp(&m1.name.to_lowercase(), &m2.name.to_lowercase()))
        .map(|mod_data| {
            format!(
                "{} {} {}\n  {}{}",
                style("-").magenta(),
                style(mod_data.name).green().bold(),
                mod_data.latest_release.unwrap().get_version(),
                style(mod_data.owner).cyan(),
                if mod_data.summary.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "\n{}",
                        textwrap::indent(
                            &textwrap::wrap(
                                mod_data.summary.lines().next().unwrap_or_default(),
                                90
                            )
                            .join("\n"),
                            "  ",
                        )
                    )
                }
            )
        })
        .join("\n\n");

    println!("{}", results);

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

    fn get_release_list(&self) -> &Vec<T>;

    fn get_release_list_mut(&mut self) -> &mut Vec<T>;

    fn remove_release(&mut self, ident: &ModIdent) -> Result<()> {
        let index = if let Some(version) = &ident.version {
            self.get_release_list()
                .iter()
                .rev()
                .enumerate()
                .find(|(_index, entry)| version == entry.get_version())
                .map(|(index, _)| index)
        } else if !self.get_release_list().is_empty() {
            Some(self.get_release_list().len() - 1)
        } else {
            None
        }
        .ok_or_else(|| anyhow!("{} not found in release list"))?;

        self.get_release_list_mut().remove(index);

        Ok(())
    }
}

trait HasVersion {
    fn get_version(&self) -> &Version;
}

trait HasDependencies {
    fn get_dependencies(&self) -> Option<&Vec<ModDependency>>;
}
