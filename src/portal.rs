use crate::config::Config;
use crate::dependency::ModDependency;
use crate::get_mod_version;
use crate::ModIdent;
use crate::Version;
use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::cmp::min;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};

// TODO: Hold authentication in this struct
pub struct Portal {
    client: Client,
    mods: HashMap<String, PortalMod>,
}

impl Portal {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            mods: HashMap::new(),
        }
    }

    // TODO: Figure out how to avoid cloning the entries
    pub fn get(&mut self, mod_name: &str) -> Result<PortalMod> {
        match self.mods.get(mod_name) {
            Some(entry) => Ok(entry.clone()),
            None => {
                let res = self
                    .client
                    .get(format!(
                        "https://mods.factorio.com/api/mods/{}/full",
                        mod_name
                    ))
                    .send()?
                    // TODO: Custom errors
                    .error_for_status()?;

                let entry: PortalMod = res.json()?;
                self.mods.insert(mod_name.to_string(), entry.clone());
                Ok(entry)
            }
        }
    }

    pub fn download(&mut self, ident: &ModIdent, config: &Config) -> Result<()> {
        // Get authentication token and username
        let auth = config
            .portal_auth
            .as_ref()
            .ok_or_else(|| anyhow!("Mod portal authentication not found"))?;

        let mod_data = self.get(&ident.name)?;
        let release_data = get_mod_version(&mod_data.releases, ident)
            .ok_or_else(|| anyhow!("{} was not found on the mod portal", ident))?;

        // Download the mod
        let mut res = match self
            .client
            .get(format!(
                "https://mods.factorio.com{}",
                release_data.download_url
            ))
            .query(&[("username", &auth.username), ("token", &auth.token)])
            .send()?
            .error_for_status()
        {
            Ok(res) => res,
            Err(err) if err.status().unwrap() == StatusCode::FORBIDDEN => {
                return Err(anyhow!("Mod portal credentials are invalid"))
            }
            Err(err) => return Err(anyhow!(err)),
        };

        let total_size = res
            .content_length()
            .ok_or_else(|| anyhow!("No content length"))?;

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{elapsed_precise:.blue}] [{bar:.green}] {bytes} / {total_bytes} ({bytes_per_sec}, {eta})")
                .progress_chars("=> "),
        );

        pb.set_message(format!(
            "{} {} v{}",
            style("Downloading").cyan().bold(),
            ident.name,
            release_data.version
        ));

        // TODO: Put this in a temp directory instead
        let path = config.mods_dir.join(format!(
            "{}_DOWNLOAD",
            release_data
                .file_name
                .strip_suffix(".zip")
                .unwrap_or(&release_data.file_name)
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
        if hasher.finalize()[..] != hex::decode(&release_data.sha1)? {
            return Err(anyhow!("Download was corrupted"));
        }

        // Rename file
        let mut proper_path = config.mods_dir.clone();
        proper_path.push(&release_data.file_name);
        fs::rename(&path, &proper_path)?;

        // Finish up
        pb.finish_and_clear();
        println!("{} {}", style("Downloaded").cyan().bold(), ident);

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalMod {
    name: String,
    title: String,
    summary: String,
    owner: String,
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

#[derive(Clone, Debug, Deserialize)]
struct PortalInfoJson {
    #[serde(default)]
    dependencies: Option<Vec<ModDependency>>,
    factorio_version: Version,
}

// TODO: These are integration tests, not unit tests :/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_fetch() {
        let mut portal = Portal::new();
        let ident = ModIdent {
            name: "EditorExtensions".to_string(),
            version: None,
        };

        // Will fetch it
        assert!(portal.get(&ident.name).is_ok());
        // Will pull it from the local store
        // TODO: We will want to find a way to diseonnect `Client` so we can ensure that this is getting it from the local database
        assert!(portal.get(&ident.name).is_ok());
    }

    // TODO: This is a full-blown integration test that needs more setup (needs `config`)
    // #[test]
    // fn download() {
    //     let mut portal = Portal::new();
    //     let ident = ModIdent {
    //         name: "EditorExtensions".to_string(),
    //         version: None,
    //     };

    //     portal.download(&ident);
    // }
}
