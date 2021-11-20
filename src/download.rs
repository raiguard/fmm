use std::cmp::min;
use std::fs::File;
use std::io::{Read, Write};

use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::config::Config;
use crate::types::ModIdent;

pub fn download_mod(client: &Client, config: &Config, mod_ident: &ModIdent) -> Result<()> {
    // Get authentication token and username
    let portal_auth = config
        .portal_auth
        .as_ref()
        .ok_or(DownloadModErr::NoPortalAuth)?;

    // Download the mod's information
    let mod_info: ModPortalResult = client
        .get(format!(
            "https://mods.factorio.com/api/mods/{}",
            mod_ident.name
        ))
        .send()?
        .json()
        .map_err(|_| DownloadModErr::ModNotFound(mod_ident.name.to_string()))?;

    // Get the corresponding release
    let release = if let Some(version_req) = &mod_ident.version_req {
        mod_info
            .releases
            .iter()
            .rev()
            .find(|release| version_req.matches(&release.version))
    } else {
        mod_info.releases.last()
    }
    .ok_or(DownloadModErr::ModNotFound(mod_ident.name.to_string()))?;

    // Download the mod
    let mut res = client
        .get(format!("https://mods.factorio.com{}", release.download_url,))
        .query(&[
            ("username", &portal_auth.username),
            ("token", &portal_auth.token),
        ])
        .send()?
        .error_for_status()?;
    let total_size = res
        .content_length()
        .ok_or(DownloadModErr::NoContentLength)?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
    .template("{msg} [{elapsed_precise:.blue}] [{bar:.green}] {bytes} / {total_bytes} ({bytes_per_sec}, {eta})")
    .progress_chars("=> "));
    pb.set_message(format!(
        "{} {} v{}",
        style("Downloading").cyan().bold(),
        mod_ident.name,
        release.version
    ));

    let mut path = config.mods_dir.clone();
    path.push(&release.file_name);
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

    let hash = hasher.finalize();

    if &hash[..] != hex::decode(&release.sha1)? {
        return Err(anyhow!(DownloadModErr::DamagedDownload));
    }

    pb.finish_and_clear();
    println!(
        "{} {} v{}",
        style("Downloaded").cyan().bold(),
        mod_ident.name,
        release.version
    );

    Ok(())
}

#[derive(Debug, Error)]
enum DownloadModErr {
    #[error("Damaged download")]
    DamagedDownload,
    #[error("Mod `{0}` was not found on the portal")]
    ModNotFound(String),
    #[error("Could not get content length")]
    NoContentLength,
    #[error("Could not find mod portal authentication")]
    NoPortalAuth,
}

#[derive(Debug, Deserialize)]
struct ModPortalResult {
    releases: Vec<ModPortalRelease>,
}

#[derive(Debug, Deserialize)]
struct ModPortalRelease {
    download_url: String,
    file_name: String,
    sha1: String,
    version: Version,
}
