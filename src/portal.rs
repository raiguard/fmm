use crate::config::Config;
use crate::dependency::ModDependency;
use crate::directory::Directory;
use crate::get_mod_version;
use crate::types::{ModEntry, ModIdent};
use crate::version::Version;
use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use sha1::{Digest, Sha1};
use std::cmp::min;
use std::fs::{self, File};
use std::io::{Read, Write};
use thiserror::Error;

pub fn download_mod(
    mod_ident: &ModIdent,
    directory: &mut Directory,
    config: &Config,
    client: &Client,
) -> Result<bool> {
    Ok(match download_mod_internal(mod_ident, config, client) {
        Ok(data) => {
            directory.add(data);
            true
        }
        Err(err) => {
            eprintln!(
                "{} Could not download {}: {}",
                style("Error:").red().bold(),
                mod_ident.name,
                err
            );
            false
        }
    })
}

fn download_mod_internal(
    mod_ident: &ModIdent,
    config: &Config,
    client: &Client,
) -> Result<(String, ModEntry)> {
    // Get authentication token and username
    let portal_auth = config
        .portal_auth
        .as_ref()
        .ok_or(DownloadModErr::CredentialsNotFound)?;

    // println!("{} {}", style("Fetching").cyan().bold(), mod_ident.name);

    // Download mod information
    let mod_info: PortalMod = client
        .get(format!(
            "https://mods.factorio.com/api/mods/{}",
            mod_ident.name
        ))
        .send()?
        .json()?;

    // Get the corresponding release
    let release =
        get_mod_version(&mod_info.releases, mod_ident).ok_or(DownloadModErr::ModNotFound)?;

    // Download the mod
    let mut res = match client
        .get(format!("https://mods.factorio.com{}", release.download_url))
        .query(&[
            ("username", &portal_auth.username),
            ("token", &portal_auth.token),
        ])
        .send()?
        .error_for_status()
    {
        Ok(res) => res,
        Err(err) if err.status().unwrap() == StatusCode::FORBIDDEN => {
            return Err(anyhow!(DownloadModErr::InvalidCredentials))
        }
        Err(err) => return Err(anyhow!(err)),
    };

    let total_size = res
        .content_length()
        .ok_or(DownloadModErr::NoContentLength)?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise:.blue}] [{bar:.green}] {bytes} / {total_bytes} ({bytes_per_sec}, {eta})")
            .progress_chars("=> "),
    );

    pb.set_message(format!(
        "{} {} v{}",
        style("Downloading").cyan().bold(),
        mod_ident.name,
        release.version
    ));

    let mut path = config.mods_dir.clone();
    path.push(format!(
        "{}_DOWNLOAD",
        release
            .file_name
            .strip_suffix(".zip")
            .unwrap_or(&release.file_name)
    ));
    let mut file = File::create(&path)?;

    let mut downloaded: u64 = 0;

    let mut buf = vec![0; 8_096];
    let mut hasher = Sha1::new();

    while downloaded < total_size {
        if let Some(bytes) = match res.read(&mut buf) {
            Ok(bytes) => Some(bytes),
            Err(err) if matches!(err.kind(), std::io::ErrorKind::Interrupted) => None,
            Err(err) => return Err(anyhow!(err)),
        } {
            file.write_all(&buf[0..bytes])?;
            hasher.update(&buf[0..bytes]);
            // Update progress bar
            downloaded = min(downloaded + (bytes as u64), total_size);
            pb.set_position(downloaded);
        }
    }

    // Verify download
    if hasher.finalize()[..] != hex::decode(&release.sha1)? {
        return Err(anyhow!(DownloadModErr::DamagedDownload));
    }

    // Rename file
    let mut proper_path = config.mods_dir.clone();
    proper_path.push(&release.file_name);
    fs::rename(&path, &proper_path)?;

    // Finish up
    pb.finish_and_clear();
    println!(
        "{} {} v{}",
        style("Downloaded").cyan().bold(),
        mod_ident.name,
        release.version
    );

    Ok((
        mod_info.name.clone(),
        ModEntry {
            entry: fs::read_dir(&config.mods_dir)?
                .filter_map(|entry| entry.ok())
                .find(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .map(|file_name| file_name == release.file_name)
                        .is_some()
                })
                .unwrap(),
            ident: ModIdent {
                name: mod_info.name,
                version: Some(release.version.clone()),
            },
        },
    ))
}

#[derive(Debug, Error)]
enum DownloadModErr {
    #[error("Could not find mod portal credentials")]
    CredentialsNotFound,
    #[error("Damaged download")]
    DamagedDownload,
    #[error("Invalid mod portal credentials")]
    InvalidCredentials,
    #[error("Mod was not found on the portal")]
    ModNotFound,
    #[error("Could not get content length")]
    NoContentLength,
}

pub fn get_dependencies(mod_ident: &ModIdent, client: &Client) -> Result<Vec<ModDependency>> {
    println!("{}", mod_ident);
    let res = client
        .get(format!(
            "https://mods.factorio.com/api/mods/{}/full",
            mod_ident.name
        ))
        .send()?;

    let status = res.status();
    if !status.is_success() {
        return Err(anyhow!(status.canonical_reason().unwrap_or("")));
    }

    let json: PortalModFull = res.json()?;

    let release = get_mod_version(&json.releases, mod_ident).ok_or_else(|| {
        anyhow!(
            "{}: Dependency requirement could not be satisfied",
            mod_ident.name
        )
    })?;

    let info_json = release
        .info_json
        .as_ref()
        .ok_or_else(|| anyhow!("API result did not contain info.json"))?;

    Ok(info_json
        .dependencies
        .as_ref()
        .ok_or_else(|| anyhow!("Mod portal did not return dependencies"))?
        .clone())
}

#[derive(Debug, Deserialize)]
struct PortalMod {
    name: String,
    releases: Vec<PortalModRelease>,
}

#[derive(Deserialize)]
struct PortalModFull {
    name: String,
    title: String,
    summary: String,
    owner: String,
    homepage: String,
    releases: Vec<PortalModRelease>,
}

#[derive(Clone, Debug, Deserialize)]
struct PortalModRelease {
    download_url: String,
    file_name: String,
    info_json: Option<PortalInfoJson>,
    sha1: String,
    version: Version,
}

impl crate::HasVersion for PortalModRelease {
    fn get_version(&self) -> &Version {
        &self.version
    }
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
struct PortalInfoJson {
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    #[serde(default)]
    dependencies: Option<Vec<ModDependency>>,
}
