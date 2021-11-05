use anyhow::Result;
use std::fs::{DirEntry, File};
use std::path::PathBuf;
use thiserror::Error;

use zip::ZipArchive;

use crate::types::ModIdent;

pub struct SaveFile {
    pub entry: DirEntry,
    pub mods: Vec<ModIdent>,
    pub path: PathBuf,
}

impl SaveFile {
    pub fn from(path: PathBuf) -> Result<Self> {
        let mut archive = ZipArchive::new(File::open(&path)?)?;
        let filename = archive
            .file_names()
            .find(|filename| filename.contains("level-init.dat"))
            .map(ToString::to_string)
            .ok_or(SaveFileErr::NoLevelDat)?;
        let file = archive.by_name(&filename)?;

        todo!()
    }
}

#[derive(Debug, Error)]
pub enum SaveFileErr {
    #[error("No level-init.dat or level.dat was found in the save file")]
    NoLevelDat,
}
