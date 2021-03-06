use crate::dat::PropertyTree;
use crate::dat::ReadFactorioDat;
use crate::mod_ident::ModIdent;
use crate::Version;
use anyhow::anyhow;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use compress::zlib;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::SeekFrom;
use std::path::PathBuf;
use zip::ZipArchive;

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
            .ok_or_else(|| anyhow!("Save file does not contain level.dat or level.dat0"))?;
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
        let version_build = cursor.read_u16::<LittleEndian>()?;

        // TODO: What are these for?
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

        // TODO: What are these for?
        cursor.seek(SeekFrom::Current(4))?;

        let startup_settings = PropertyTree::load(&mut cursor)?;

        Ok(Self {
            mods,
            map_version: Version::new(
                version_major as u32,
                version_minor as u32,
                version_patch as u32,
                Some(version_build as u32),
            ),
            path,
            startup_settings,
        })
    }
}
