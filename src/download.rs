use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::types::{InfoJson, ModEntry, ModIdent};

pub fn download_mod(client: &Client, mod_ident: &ModIdent) -> Result<ModEntry> {
    let mod_info: ModPortalResult = serde_json::from_str(
        &client
            .get(format!(
                "https://mods.factorio.com/api/mods/{}",
                mod_ident.name
            ))
            .send()?
            .text()?,
    )?;

    println!("{:#?}", mod_info);

    todo!()
}

#[derive(Debug, Deserialize)]
struct ModPortalResult {
    // downloads_count: u32,
    name: String,
    // owner: String,
    releases: Vec<ModPortalRelease>,
    // summary: String,
    // title: String,
    // category: Option<ModPortalTag>,
}

#[derive(Debug, Deserialize)]
struct ModPortalRelease {
    download_url: String,
    file_name: String,
    version: String,
    sha1: String,
}
