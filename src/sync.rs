use anyhow::anyhow;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use compress::zlib;
use semver::Version;
use semver::VersionReq;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::SeekFrom;
use std::path::PathBuf;
use thiserror::Error;
use zip::ZipArchive;

use crate::types::ModIdent;

const READ_SIZE: usize = 1_048_576;

pub struct SaveFile {
    pub map_version: Version,
    pub mods: Vec<ModIdent>,
    pub path: PathBuf,
    pub scenario_mod_name: String,
    pub scenario: String,
    pub startup_settings: PropertyTree,
}

pub type SaveFileReader = Cursor<Vec<u8>>;

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

        cursor.seek(SeekFrom::Current(2))?;

        let scenario_name = read_string(&mut cursor)?;
        let scenario_mod_name = read_string(&mut cursor)?;

        // TODO: Handle campaigns
        cursor.seek(SeekFrom::Current(14))?;

        let num_mods = cursor.read_u8()?;

        let mut mods = Vec::with_capacity(num_mods as usize);
        for _ in 0..num_mods {
            mods.push(read_mod(&mut cursor)?);
        }

        cursor.seek(SeekFrom::Current(4));

        let startup_settings = PropertyTree::load(&mut cursor)?;

        println!("{:#?}", startup_settings);

        Ok(Self {
            mods,
            map_version: Version::new(
                version_major as u64,
                version_minor as u64,
                version_patch as u64,
            ),
            path,
            scenario: scenario_name,
            scenario_mod_name,
            startup_settings,
        })
    }
}

#[derive(Debug, Error)]
pub enum SaveFileErr {
    #[error("No level.dat was found in the save file")]
    NoLevelDat,
}

#[derive(Debug)]
pub enum PropertyTree {
    None,
    Boolean(bool),
    Number(f64),
    String(Option<String>),
    List(Vec<PropertyTree>),
    Dictionary(HashMap<String, PropertyTree>),
}

impl PropertyTree {
    pub fn load(reader: &mut SaveFileReader) -> Result<Self> {
        let pt_type = reader.read_u8()?;
        // Internal flag that we don't have to care about
        reader.seek(SeekFrom::Current(1))?;
        match pt_type {
            0 => Ok(Self::None),
            1 => Ok(Self::Boolean(read_bool(reader)?)),
            2 => Ok(Self::Number(reader.read_f64::<LittleEndian>()?)),
            3 => Ok(Self::String(read_pt_string(reader)?)),
            4 => {
                let length = reader.read_u32::<LittleEndian>()?;
                let mut list = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    read_pt_string(reader)?;
                    list.push(PropertyTree::load(reader)?);
                }
                Ok(Self::List(list))
            }
            5 => {
                let length = reader.read_u32::<LittleEndian>()?;
                let mut dict = HashMap::with_capacity(length as usize);
                for _ in 0..length {
                    dict.insert(
                        read_pt_string(reader)?
                            .ok_or_else(|| anyhow!("Missing key in PropertyTree Dictionary"))?,
                        PropertyTree::load(reader)?,
                    );
                }
                Ok(Self::Dictionary(dict))
            }
            _ => Err(anyhow!("Invalid data type in PropertyTree: {}", pt_type)),
        }
    }
}

fn read_bool(reader: &mut SaveFileReader) -> Result<bool> {
    match reader.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(anyhow!("Invalid boolean representation in PropertyTree.")),
    }
}

fn read_string(reader: &mut SaveFileReader) -> Result<String> {
    let scenario_len = reader.read_u8()?;
    let mut scenario_name = vec![0; scenario_len as usize];
    reader.read_exact(&mut scenario_name)?;

    Ok(String::from_utf8_lossy(&scenario_name).to_string())
}

// Strings in PropertyTrees have an extra byte to inform us if they are empty
fn read_pt_string(reader: &mut SaveFileReader) -> Result<Option<String>> {
    if read_bool(reader)? {
        Ok(None)
    } else {
        read_string(reader).map(Some)
    }
}

fn read_u16_optimized(reader: &mut SaveFileReader) -> Result<u16> {
    let mut version: u16 = reader.read_u8()?.into();
    if version == 255 {
        version = reader.read_u16::<LittleEndian>()?;
    }
    Ok(version)
}

fn read_mod(reader: &mut SaveFileReader) -> Result<ModIdent> {
    let mod_name = read_string(reader)?;

    let version_major = read_u16_optimized(reader)?;
    let version_minor = read_u16_optimized(reader)?;
    let version_patch = read_u16_optimized(reader)?;

    // We don't care about the CRC
    let _crc = reader.read_u32::<LittleEndian>()?;

    Ok(ModIdent {
        name: mod_name,
        version_req: Some(VersionReq::exact(&Version::new(
            version_major as u64,
            version_minor as u64,
            version_patch as u64,
        ))),
    })
}
