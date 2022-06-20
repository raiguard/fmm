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

use crate::cli::{Args, Cmd};
use crate::config::Config;
use crate::dependency::ModDependency;
use crate::directory::Directory;
use crate::mod_ident::ModIdent;
use crate::portal::Portal;
use crate::save_file::SaveFile;
use crate::version::Version;
use anyhow::{anyhow, bail, Context, Result};
use console::style;
use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn run(args: Args) -> Result<()> {
    let config = Config::new(args)?;

    match &config.cmd {
        Cmd::Disable { mods } => disable(&config, mods),
        Cmd::Download { mods } => download(&config, mods),
        Cmd::Enable { ignore_deps, mods } => enable(&config, mods),
        Cmd::Query { mods } => query(&config, mods),
        Cmd::Remove { mods } => remove(&config, mods),
        Cmd::Search { query } => search(&config, query),
        Cmd::Sync {
            is_set,
            no_download,
            preserve,
            arg,
        } => {
            let mut directory = Directory::new(&config.mods_dir)?;

            if !preserve {
                directory.disable_all();
            }

            let mods = if *is_set {
                if let Some(sets) = &config.sets {
                    if let Some(set) = sets.get(arg) {
                        set.clone()
                    } else {
                        bail!("mod set '{}' does not exist", arg);
                    }
                } else {
                    bail!("no mod sets have been defined");
                }
            } else {
                let path = PathBuf::from_str(arg)?;
                if !path.exists() {
                    bail!("given path does not exist");
                }
                let save_file = SaveFile::from(path)?;
                // Sync startup settings
                directory
                    .settings
                    .merge_startup_settings(&save_file.startup_settings)?;
                // Extract mods to enable or download
                save_file
                    .mods
                    .iter()
                    .filter(|ident| ident.name != "base")
                    .cloned()
                    .collect()
            };

            // Download mods that we don't have
            if !no_download {
                let to_download: Vec<ModIdent> = mods
                    .iter()
                    .cloned()
                    .filter(|ident| !directory.contains(ident))
                    .collect();
                download(&config, &to_download)?;
            }

            // Enable mods
            enable(&config, &mods)?;

            Ok(())
        }
        Cmd::Update { mods } => update(&config, mods),
        Cmd::Upload { file } => upload(&config, file),
    }
}

fn disable(config: &Config, mods: &[ModIdent]) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;
    if mods.is_empty() {
        directory.disable_all();
    } else {
        for ident in mods {
            directory.disable(ident);
        }
    }
    directory.save()?;
    Ok(())
}

fn download(config: &Config, mods: &[ModIdent]) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;
    let mut portal = Portal::new();

    for ident in mods {
        if directory.contains(ident) {
            eprintln!("{} is already downloaded, use --force to override", ident);
            continue;
        }
        match portal.download(ident, config) {
            Ok((ident, path)) => {
                directory.add(ident, path);
            }
            Err(e) => eprintln!("failed to download mod: {}", e),
        }
    }

    Ok(())
}

fn enable(config: &Config, mods: &[ModIdent]) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;

    for ident in mods {
        if let Err(e) = directory.enable(ident) {
            eprintln!("error: {}", e);
        }
    }

    directory.save()?;

    Ok(())
}

fn query(config: &Config, mods: &[ModIdent]) -> Result<()> {
    let directory = Directory::new(&config.mods_dir)?;
    for ident in mods {
        match directory.get(ident) {
            Some(entry) => {
                for release in entry.get_release_list() {
                    if ident.version.is_none()
                        || release.get_version() == ident.version.as_ref().unwrap()
                    {
                        println!("{} {}", ident.name, release.get_version());
                    }
                }
            }
            None => eprintln!("error: mod '{}' not found", ident),
        }
    }

    Ok(())
}

fn remove(config: &Config, mods: &[ModIdent]) -> Result<()> {
    let mut directory = Directory::new(&config.mods_dir)?;
    for ident in mods {
        directory.remove(ident)?;
    }
    Ok(())
}

fn search(_config: &Config, query: &str) -> Result<()> {
    let portal = Portal::new();

    let mod_list = portal.get_all_mods()?;

    let query = query.to_lowercase();

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

fn update(config: &Config, mods: &[String]) -> Result<()> {
    let directory = Directory::new(&config.mods_dir)?;
    let portal = Portal::new();

    let no_input = mods.is_empty();

    let latest_portal: Vec<ModIdent> = portal
        .get_all_mods()?
        .results
        .iter()
        .filter_map(|entry| {
            entry.latest_release.as_ref().map(|release| ModIdent {
                name: entry.name.clone(),
                version: Some(release.version.clone()),
            })
        })
        .collect();

    let latest_local: Vec<ModIdent> = if no_input {
        directory.get_all_names()
    } else {
        mods.to_vec()
    }
    .iter()
    .filter_map(|name| {
        if let Some(latest_version) = directory.get_newest(name) {
            Some(ModIdent {
                name: name.to_string(),
                version: Some(latest_version.get_version().clone()),
            })
        } else {
            eprintln!("mod '{}' was not found in the mods directory", name);
            None
        }
    })
    .collect();

    let to_download: Vec<ModIdent> = latest_local
        .iter()
        .filter_map(|ident| {
            if let Some(pident) = latest_portal
                .iter()
                .find(|pident| pident.name == ident.name)
            {
                if ident.get_guaranteed_version() < pident.get_guaranteed_version() {
                    Some(pident)
                } else {
                    // Printing this message when updating all mods is unnecessary
                    if !no_input {
                        eprintln!(
                            "mod '{}' is already up-to-date (local: {}, portal: {})",
                            ident.name,
                            ident.get_guaranteed_version(),
                            pident.get_guaranteed_version()
                        );
                    }
                    None
                }
            } else {
                eprintln!("mod '{}' was not found on the mod portal", ident.name);
                None
            }
        })
        .cloned()
        .collect();

    if !to_download.is_empty() {
        download(config, &to_download)?;
    } else {
        eprintln!("there is nothing to do");
    }

    Ok(())
}

fn upload(config: &Config, file: &Path) -> Result<()> {
    let portal = Portal::new();

    portal
        .upload(config, file)
        .context("Failed to upload mod")?;

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
        .ok_or_else(|| anyhow!("{} not found in release list", ident))?;

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
