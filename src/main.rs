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
use dependency::ModDependency;
use directory::{Directory, WrappedDirectory};
use itertools::Itertools;
use mod_ident::ModIdent;
use pico_args::Arguments;
use portal::{Portal, WrappedPortal};
use save_file::SaveFile;
use std::path::PathBuf;
use std::str::FromStr;
use version::Version;

const HELP: &str = "usage: fmm <options> <subcommand>
options:
    --game-dir <PATH>  custom game directory path
    --mods-dir <PATH>  custom mod directory path
    --token <TOKEN>    oauth token for the mod portal
subcommands:
    disable <MODS>
    download <MODS>
    enable <MODS>
    query <MODS>
    remove <MODS>
    search <QUERY>
    sync (-s --set) <SET/FILE>
    update <MODS>
    upload <PATH>";

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

    }
}

pub fn main() -> Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("--help") || std::env::args().len() == 1 {
        println!("{HELP}");
        return Ok(());
    }

    let config = Config::new(&mut args)?;
    let mut ctx = Ctx::new(&config);

    match args.subcommand()?.as_deref() {
        Some("disable") => disable(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("download") => download(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("enable") => enable(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("query") => query(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("remove") => remove(&mut ctx, &config, &finish_args::<ModIdent>(args)?)?,
        Some("search") => search(&mut ctx, &config, args.free_from_str()?)?,
        Some("sync") => sync(
            &mut ctx,
            &config,
            &args.contains(["-s", "--sync"]),
            &args.free_from_str()?,
        )?,
        Some("update") => update(&mut ctx, &config, &finish_args::<String>(args)?)?,
        Some("upload") => upload(&mut ctx, &config, args.free_from_str()?)?,
        Some(cmd) => eprintln!("unknown subcommand: {cmd}\n{HELP}"),
        None => eprintln!("{HELP}"),
    };

    Ok(())
}

fn disable(ctx: &mut Ctx, _config: &Config, mods: &[ModIdent]) -> Result<()> {
    if mods.is_empty() {
        ctx.directory.get().disable_all();
    } else {
        for ident in mods {
            ctx.directory.get().disable(ident);
        }
    }
    ctx.directory.get().save()?;
    Ok(())
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

    ctx.directory.get().save()?;

    Ok(())
}

fn enable(ctx: &mut Ctx, config: &Config, mods: &[ModIdent]) -> Result<()> {
    let to_enable = mods.to_vec();
    let to_check = mods.to_vec();

    for ident in mods {
        if let Err(e) = ctx.directory.get().enable(&ident) {
            eprintln!("could not enable mod: {}", e);
        }
    }

    ctx.directory.get().save()?;

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
    ctx.directory.get().save()?;
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

fn sync(ctx: &mut Ctx, config: &Config, is_set: &bool, arg: &String) -> Result<()> {
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
            bail!("path '{}' does not exist", path.to_str().unwrap());
        }
        let save_file = SaveFile::from(path)?;
        // Sync startup settings
        ctx.directory
            .get()
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
    if !config.sync_no_download {
        let directory = ctx.directory.get();
        let to_download: Vec<ModIdent> = mods
            .iter()
            .cloned()
            .filter(|ident| !directory.contains(ident))
            .collect();
        download(ctx, config, &to_download)?;
    }

    ctx.directory.get().disable_all();

    ctx.directory.get().save()?;

    // Enable mods
    enable(ctx, config, &mods)?;

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

    ctx.directory.get().save()?;

    Ok(())
}

fn upload(ctx: &mut Ctx, config: &Config, file: PathBuf) -> Result<()> {
    ctx.portal
        .get()
        .upload(config, &file)
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
