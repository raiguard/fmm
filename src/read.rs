use anyhow::anyhow;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use semver::Version;
use semver::VersionReq;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::Read;
use std::io::SeekFrom;

use crate::types::ModIdent;

pub type DatReader = Cursor<Vec<u8>>;

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
    pub fn load(reader: &mut DatReader) -> Result<Self> {
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

pub fn read_bool(reader: &mut DatReader) -> Result<bool> {
    match reader.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(anyhow!("Invalid boolean representation in PropertyTree.")),
    }
}

pub fn read_string(reader: &mut DatReader) -> Result<String> {
    let scenario_len = reader.read_u8()?;
    let mut scenario_name = vec![0; scenario_len as usize];
    reader.read_exact(&mut scenario_name)?;

    Ok(String::from_utf8_lossy(&scenario_name).to_string())
}

// Strings in PropertyTrees have an extra byte to inform us if they are empty
pub fn read_pt_string(reader: &mut DatReader) -> Result<Option<String>> {
    if read_bool(reader)? {
        Ok(None)
    } else {
        read_string(reader).map(Some)
    }
}

pub fn read_u16_optimized(reader: &mut DatReader) -> Result<u16> {
    let mut version: u16 = reader.read_u8()?.into();
    if version == 255 {
        version = reader.read_u16::<LittleEndian>()?;
    }
    Ok(version)
}

pub fn read_mod(reader: &mut DatReader) -> Result<ModIdent> {
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
