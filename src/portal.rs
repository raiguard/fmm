use crate::dependency::ModDependency;
use crate::{Config, HasDependencies, HasReleases, HasVersion, ModIdent, Version};
use anyhow::{anyhow, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::cmp::min;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

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

    pub fn contains(&self, mod_name: &str) -> bool {
        self.mods.contains_key(mod_name)
    }

    pub fn fetch(&mut self, mod_name: &str) {
        if let Err(e) = self.fetch_internal(mod_name) {
            eprintln!(
                "{} could not fetch {}: {}",
                style("Error:").red().bold(),
                mod_name,
                e
            );
        }
    }

    fn fetch_internal(&mut self, mod_name: &str) -> Result<()> {
        println!("{} {}", style("Fetching").cyan().bold(), mod_name);
        let res = self
            .client
            .get(format!(
                "https://mods.factorio.com/api/mods/{}/full",
                mod_name
            ))
            .send()?
            // TODO: Custom errors
            .error_for_status()?;

        self.mods.insert(mod_name.to_string(), res.json()?);

        Ok(())
    }

    pub fn get(&self, mod_name: &str) -> Option<&PortalMod> {
        self.mods.get(mod_name)
    }

    pub fn get_or_fetch(&mut self, mod_name: &str) -> Option<&PortalMod> {
        if !self.contains(mod_name) {
            self.fetch(mod_name);
        }
        self.get(mod_name)
    }

    pub fn download(&mut self, ident: &ModIdent, config: &Config) -> Result<(ModIdent, PathBuf)> {
        // Get authentication token and username
        let auth = config
            .portal_auth
            .as_ref()
            .ok_or_else(|| anyhow!("Mod portal authentication not found"))?;

        if !self.contains(&ident.name) {
            self.fetch(&ident.name);
        }
        let mod_data = self
            .get(&ident.name)
            .ok_or_else(|| anyhow!("Cannot download {}", ident))?;
        let release_data = mod_data
            .get_release(ident)
            .ok_or_else(|| anyhow!("{} was not found on the mod portal", ident))?;

        let ident = ModIdent {
            name: ident.name.clone(),
            version: Some(release_data.get_version().clone()),
        };

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

        pb.set_message(format!("{} {}", style("Downloading").cyan().bold(), ident,));

        let mut file = NamedTempFile::new()?;

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
        let proper_path = config.mods_dir.join(&release_data.file_name);
        std::fs::copy(file.path(), &proper_path)?;

        // Finish up
        pb.finish_and_clear();
        println!("{} {}", style("Downloaded").cyan().bold(), ident);

        Ok((ident, proper_path))
    }

    pub fn get_or_fetch_newest_matching(
        &mut self,
        dependency: &ModDependency,
    ) -> Option<&PortalModRelease> {
        self.get_or_fetch(&dependency.name).and_then(|mod_data| {
            if let Some(version_req) = &dependency.version_req {
                mod_data
                    .releases
                    .iter()
                    .rev()
                    .find(|release| version_req.matches(release.get_version()))
            } else {
                mod_data.releases.last()
            }
        })
    }

    pub fn get_all_mods(&self) -> Result<PortalAllRes> {
        Ok(self
            .client
            .get("https://mods.factorio.com/api/mods?page_size=max")
            .send()?
            .json()?)
    }

    pub fn upload(&self, config: &Config, file: &Path) -> Result<()> {
        let upload_token = config
            .upload_token
            .as_ref()
            .ok_or_else(|| anyhow!("Missing mod portal upload token"))?;

        let file_name = file
            .file_name()
            .ok_or_else(|| anyhow!("Unable to parse filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Filename must be valid unicode"))?;

        let (name, _) = file_name.rsplit_once('_').ok_or_else(|| {
            anyhow!("Invalid mod filename, must be formatted as 'modname_version.zip'")
        })?;

        println!("Uploading {}", file_name);

        let form = Form::new().text("mod", name.to_string());

        let res = self
            .client
            .post("https://mods.factorio.com/api/v2/mods/releases/init_upload")
            .header("Authorization", format!("Bearer {upload_token}"))
            .multipart(form)
            .send()?;

        let body: InitUploadRes = res.json()?;
        match body {
            InitUploadRes::Success { upload_url } => {
                let form = Form::new().part("file", Part::file(file)?);
                let res = self.client.post(upload_url).multipart(form).send()?;

                let body: FinishUploadRes = res.json()?;
                match body {
                    FinishUploadRes::Failure {
                        error: _error,
                        message,
                    } => return Err(anyhow!(message)),
                    FinishUploadRes::Success { success: _success } => println!("Upload successful"),
                }
            }
            InitUploadRes::Failure {
                error: _error,
                message,
            } => return Err(anyhow!(message)),
        }

        Ok(())
    }
}

pub struct WrappedPortal {
    inner: Option<Portal>,
}

impl WrappedPortal {
    pub fn new(_config: &Config) -> Self {
        Self { inner: None }
    }

    pub fn get(&mut self) -> &mut Portal {
        if self.inner.is_none() {
            self.inner = Some(Portal::new());
        }
        self.inner.as_mut().unwrap()
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InitUploadRes {
    Failure { error: String, message: String },
    Success { upload_url: String },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FinishUploadRes {
    Failure { error: String, message: String },
    Success { success: bool },
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalMod {
    releases: Vec<PortalModRelease>,
}

impl HasReleases<PortalModRelease> for PortalMod {
    fn get_release_list(&self) -> &Vec<PortalModRelease> {
        &self.releases
    }

    fn get_release_list_mut(&mut self) -> &mut Vec<PortalModRelease> {
        &mut self.releases
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalModRelease {
    pub download_url: String,
    pub file_name: String,
    pub info_json: Option<PortalInfoJson>,
    pub sha1: String,
    pub version: Version,
}

impl HasDependencies for PortalModRelease {
    fn get_dependencies(&self) -> Option<&Vec<ModDependency>> {
        self.info_json
            .as_ref()
            .and_then(|info_json| info_json.dependencies.as_ref())
    }
}

impl HasVersion for PortalModRelease {
    fn get_version(&self) -> &Version {
        &self.version
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalInfoJson {
    #[serde(default)]
    pub dependencies: Option<Vec<ModDependency>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalAllRes {
    pub results: Vec<PortalAllMod>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PortalAllMod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub owner: String,
    pub latest_release: Option<PortalModRelease>,
}

// TODO: These are integration tests, not unit tests :/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_methods() {
        let mut portal = Portal::new();
        let ident = ModIdent {
            name: "EditorExtensions".to_string(),
            version: None,
        };

        // Mod does not exist
        assert!(!portal.contains(&ident.name));
        assert!(portal.get(&ident.name).is_none());

        // Fetch mod
        portal.fetch(&ident.name);

        // Mod exists
        assert!(portal.contains(&ident.name));
        assert!(portal.get(&ident.name).is_some());
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
