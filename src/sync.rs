use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use compress::zlib;
use semver::Version;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::SeekFrom;
use std::path::PathBuf;
use thiserror::Error;
use zip::ZipArchive;

use crate::read::PropertyTree;
use crate::read::ReadFactorioDat;
use crate::types::ModIdent;

const READ_SIZE: usize = 1_048_576;

pub struct SaveFile {
    pub map_version: Version,
    pub mods: Vec<ModIdent>,
    pub path: PathBuf,
    pub startup_settings: PropertyTree,
}

impl SaveFile {
    pub fn from(path: PathBuf) -> Result<Self> {
        println!("Reading save file...");
        let mut archive = ZipArchive::new(File::open(&path)?)?;
        let mut compressed = true;
        let filenames: Vec<&str> = archive.file_names().collect();
        let filename = filenames
            .iter()
            .find(|filename| filename.contains("level.dat0"))
            .or_else(|| {
                compressed = false;
                filenames
                    .iter()
                    .find(|filename| filename.contains("level.dat"))
            })
            .map(ToString::to_string)
            .ok_or(SaveFileErr::NoLevelDat)?;
        let file = archive.by_name(&filename)?;

        let decompressed = if compressed {
            let mut bytes = Vec::with_capacity(READ_SIZE);
            zlib::Decoder::new(file).read_to_end(&mut bytes)?;
            bytes
        } else {
            file.bytes()
                .take(READ_SIZE)
                .filter_map(|byte| byte.ok())
                .collect()
        };

        let mut cursor = Cursor::new(decompressed);
        let version_major = cursor.read_u16::<LittleEndian>()?;
        let version_minor = cursor.read_u16::<LittleEndian>()?;
        let version_patch = cursor.read_u16::<LittleEndian>()?;
        let _version_build = cursor.read_u16::<LittleEndian>()?;

        // What are these for?
        cursor.seek(SeekFrom::Current(2))?;

        let _scenario_name = cursor.read_string()?;
        let _scenario_mod_name = cursor.read_string()?;

        // TODO: Handle campaigns
        cursor.seek(SeekFrom::Current(14))?;

        let num_mods = cursor.read_u8()?;

        let mut mods = Vec::with_capacity(num_mods as usize);
        for _ in 0..num_mods {
            mods.push(cursor.read_mod()?);
        }

        // What are these for?
        cursor.seek(SeekFrom::Current(4))?;

        let startup_settings = PropertyTree::load(&mut cursor)?;

        Ok(Self {
            mods,
            map_version: Version::new(
                version_major as u64,
                version_minor as u64,
                version_patch as u64,
            ),
            path,
            startup_settings,
        })
    }
}

#[derive(Debug, Error)]
pub enum SaveFileErr {
    #[error("No level.dat was found in the save file")]
    NoLevelDat,
}
