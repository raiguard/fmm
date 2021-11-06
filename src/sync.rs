use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use compress::zlib;
use std::fs::{DirEntry, File};
use std::io::Cursor;
use std::io::Read;
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
            .find(|filename| filename.contains("level.dat0"))
            .map(ToString::to_string)
            .ok_or(SaveFileErr::NoLevelDat)?;
        let file = archive.by_name(&filename)?;

        let mut decompressed = Vec::new();
        zlib::Decoder::new(file).read_to_end(&mut decompressed)?;

        let mut reader = Cursor::new(decompressed);
        let version_major = reader.read_u16::<LittleEndian>()?;
        let version_minor = reader.read_u16::<LittleEndian>()?;
        let version_patch = reader.read_u16::<LittleEndian>()?;
        let version_build = reader.read_u16::<LittleEndian>()?;

        println!(
            "{}.{}.{}.{}",
            version_major, version_minor, version_patch, version_build
        );

        // Factorio level.dat format
        // First eight bytes are the map version
        // Then magic
        // Then mods

        todo!()
    }
}

#[derive(Debug, Error)]
pub enum SaveFileErr {
    #[error("No level-init.dat was found in the save file")]
    NoLevelDat,
}
