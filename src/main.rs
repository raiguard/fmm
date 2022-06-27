#![allow(unstable_name_collisions)]

mod config;
mod dat;
mod dependency;
mod directory;
mod mod_ident;
mod mod_settings;
mod portal;
mod save_file;
mod version;

use anyhow::{anyhow, bail, Context, Result};
use config::Config;
use console::style;
use dependency::{ModDependency, ModDependencyType};
use directory::WrappedDirectory;
use itertools::Itertools;
use mod_ident::ModIdent;
use pico_args::Arguments;
use portal::WrappedPortal;
use save_file::SaveFile;
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use version::Version;

const HELP: &str = "usage: fmm <options> <subcommand>
options:
    --config <PATH>    path to a custom configuration file
    --game-dir <PATH>  path to the game directory
    --mods-dir <PATH>  path to the mods directory
    --token <TOKEN>    oauth token for the mod portal
subcommands:
    disable <MODS>    disable the given mods, or all mods if no mods are given
    download <MODS>   download the given mods
    enable <MODS>     enable the given mods
    enable-set <SET>  enable the mods from the given mod set
    query <MODS>      query the local mods folder
    remove <MODS>     remove the given mods
    search <QUERY>    search for mods on the mod portal
    sync <MODS>       enable the given mods, downloading if necessary
    sync-file <PATH>  enable the mods from the given save file, downloading if necessary
    sync-set <SET>    enable the mods from the given mod set, downloading if necessary
    update <MODS>     update the given mods, or all mods if no mods are given
    upload <PATH>     upload the given mod zip file to the mod portal";

pub fn main() -> Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("--help") || std::env::args().len() == 1 {
        println!("{HELP}");
        return Ok(());
    }

    let config = Config::new(&mut args)?;
    let mut ctx = Ctx::new(&config);

    match args.subcommand()?.as_deref() {
        Some("disable") => disable(&mut ctx, &config, &finish_args::<ModIdent>(args)?),
        Some("download") => download(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("enable") => {
            let mods = ctx.add_dependencies(finish_args::<ModIdent>(args)?, false);
            enable(&mut ctx, &config, mods)?
        }
        Some("enable-set") => {
            let mods = ctx.add_dependencies(config.extract_mod_set(&args.free_from_str()?)?, false);
            enable(&mut ctx, &config, mods)?
        }
        Some("query") => query(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("remove") => remove(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("search") => search(&mut ctx, &config, args.free_from_str()?)?,
        Some("sync") => {
            let mods = ctx.add_dependencies(finish_args::<ModIdent>(args)?, true);
            sync(&mut ctx, &config, mods)?
        }
        Some("sync-file") => sync_file(&mut ctx, &config, args.free_from_str()?)?,
        Some("sync-set") => {
            let mods = ctx.add_dependencies(config.extract_mod_set(&args.free_from_str()?)?, true);
            sync(&mut ctx, &config, mods)?
        }
        Some("update") => update(&mut ctx, &config, &finish_args::<String>(args)?)?,
        Some("upload") => upload(&mut ctx, &config, args.free_from_str()?)?,
        Some(cmd) => eprintln!("unknown subcommand: {cmd}\n{HELP}"),
        None => eprintln!("{HELP}"),
    };

    if ctx.directory.is_some() {
        ctx.directory.get().save()?;
    }

    Ok(())
}

fn disable(ctx: &mut Ctx, _config: &Config, mods: &[ModIdent]) {
    if mods.is_empty() {
        ctx.directory.get().disable_all();
    } else {
        for ident in mods {
            ctx.directory.get().disable(ident);
        }
    }
}

fn download(ctx: &mut Ctx, config: &Config, mods: &[ModIdent]) -> Result<()> {
    for ident in mods {
        if ctx.directory.get().contains(ident) {
            eprintln!("{} is already downloaded, use --force to override", ident);
            continue;
        }
        match ctx.portal.get().download(ident, config) {
            Ok((ident, path)) => {
                ctx.directory.get().add(ident, path);
            }
            Err(e) => eprintln!("failed to download mod: {}", e),
        }
    }
    Ok(())
}

fn enable(ctx: &mut Ctx, _config: &Config, mods: Vec<ModIdent>) -> Result<()> {
    for ident in mods {
        if let Err(e) = ctx.directory.get().enable(&ident) {
            eprintln!("could not enable mod: {}", e);
        }
    }

    Ok(())
}

fn query(ctx: &mut Ctx, _config: &Config, mods: &[ModIdent]) -> Result<()> {
    for ident in mods {
        match ctx.directory.get().get(ident) {
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

fn remove(ctx: &mut Ctx, _config: &Config, mods: &[ModIdent]) -> Result<()> {
    for ident in mods {
        ctx.directory.get().remove(ident)?;
    }
    Ok(())
}

fn search(ctx: &mut Ctx, _config: &Config, query: String) -> Result<()> {
    let mod_list = ctx.portal.get().get_all_mods()?;

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

fn sync(ctx: &mut Ctx, config: &Config, mods: Vec<ModIdent>) -> Result<()> {
    let to_download: Vec<ModIdent> = mods
        .iter()
        .cloned()
        .filter(|ident| !ctx.directory.get().contains(ident))
        .collect();
    download(ctx, config, &to_download)?;

    ctx.directory.get().disable_all();

    enable(ctx, config, mods)?;

    Ok(())
}

fn update(ctx: &mut Ctx, config: &Config, mods: &[String]) -> Result<()> {
    let no_input = mods.is_empty();

    let latest_portal: Vec<ModIdent> = ctx
        .portal
        .get()
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
        ctx.directory.get().get_all_names()
    } else {
        mods.to_vec()
    }
    .iter()
    .filter_map(|name| {
        if let Some(latest_version) = ctx.directory.get().get_newest(name) {
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
        download(ctx, config, &to_download)?;
    } else {
        eprintln!("there is nothing to do");
    }

    Ok(())
}

fn sync_file(ctx: &mut Ctx, config: &Config, path: PathBuf) -> Result<()> {
    if !path.exists() {
        bail!("path '{}' does not exist", path.to_str().unwrap());
    }
    let file = SaveFile::from(path)?;
    // Sync startup settings
    ctx.directory
        .get()
        .settings
        .merge_startup_settings(&file.startup_settings)?;
    // Extract mods to enable or download
    let mods: Vec<ModIdent> = file
        .mods
        .iter()
        .filter(|ident| ident.name != "base")
        .cloned()
        .map(|mut ident| {
            if config.sync_latest_versions {
                ident.version = None;
            }
            ident
        })
        .collect();

    // Latest versions may have different dependency requirements than the versions in the save
    let mods = if config.sync_latest_versions {
        ctx.add_dependencies(mods, true)
    } else {
        mods
    };

    sync(ctx, config, mods)
}

fn upload(ctx: &mut Ctx, config: &Config, file: PathBuf) -> Result<()> {
    ctx.portal
        .get()
        .upload(config, &file)
        .context("Failed to upload mod")?;

    Ok(())
}

pub struct Ctx {
    pub directory: WrappedDirectory,
    pub portal: WrappedPortal,
}

impl Ctx {
    pub fn new(config: &Config) -> Self {
        Self {
            directory: WrappedDirectory::new(config),
            portal: WrappedPortal::new(config),
        }
    }

    pub fn add_dependencies(
        &mut self,
        mut to_check: Vec<ModIdent>,
        check_portal: bool,
    ) -> Vec<ModIdent> {
        let mut mods = vec![];
        let mut checked = HashSet::new();

        while !to_check.is_empty() {
            let mut to_check_next = vec![];
            for ident in &to_check {
                if !checked.contains(&ident.name) {
                    checked.insert(ident.name.clone());
                    mods.push(ident.clone());
                    self.directory
                        .get()
                        .get(ident)
                        .and_then(|entry| {
                            entry
                                .get_release(ident)
                                .and_then(|release| release.get_dependencies())
                        })
                        .or_else(|| {
                            self.portal
                                .get()
                                .get_or_fetch(&ident.name)
                                .and_then(|entry| {
                                    entry
                                        .get_release(ident)
                                        .and_then(|release| release.get_dependencies())
                                })
                        })
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .filter(|dependency| {
                            dependency.name != "base"
                                && !checked.contains(&dependency.name)
                                && matches!(
                                    dependency.dep_type,
                                    ModDependencyType::Required | ModDependencyType::NoLoadOrder
                                )
                        })
                        .filter_map(|dependency| {
                            let mut newest = None;
                            if let Some(entry) =
                                self.directory.get().get_newest_matching(dependency)
                            {
                                newest = Some(entry.ident.clone());
                            };

                            if newest.is_none() && check_portal {
                                newest = self
                                    .portal
                                    .get()
                                    .get_or_fetch_newest_matching(dependency)
                                    .map(|release| ModIdent {
                                        name: dependency.name.clone(),
                                        version: Some(release.version.clone()),
                                    });
                            }

                            if newest.is_none() {
                                eprintln!(
                                    "no mod found that satisfies dependency '{}'",
                                    dependency
                                );
                            }

                            newest
                        })
                        .for_each(|ident| {
                            to_check_next.push(ident);
                        });
                }
            }
            to_check = to_check_next;
        }

        mods
    }
}

fn finish_args<T>(args: Arguments) -> Result<Vec<T>>
where
    T: FromStr,
    anyhow::Error: From<<T as FromStr>::Err>,
{
    args.finish()
        .iter()
        .map(|str| {
            str.to_str()
                .ok_or_else(|| anyhow!("argument '{:?}' is not valid unicode", str))
                .and_then(|str| T::from_str(str).map_err(|e| anyhow!(e)))
        })
        .collect()
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
