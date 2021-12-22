use anyhow::anyhow;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use semver::Version;
use semver::VersionReq;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
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
            1 => Ok(Self::Boolean(reader.read_bool()?)),
            2 => Ok(Self::Number(reader.read_f64::<LittleEndian>()?)),
            3 => Ok(Self::String(reader.read_pt_string()?)),
            4 => {
                let length = reader.read_u32::<LittleEndian>()?;
                let mut list = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    reader.read_pt_string()?;
                    list.push(PropertyTree::load(reader)?);
                }
                Ok(Self::List(list))
            }
            5 => {
                let length = reader.read_u32::<LittleEndian>()?;
                let mut dict = HashMap::with_capacity(length as usize);
                for _ in 0..length {
                    dict.insert(
                        reader
                            .read_pt_string()?
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

pub trait ReadFactorioDat: io::Read {
    fn read_bool(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(anyhow!("Invalid boolean representation in PropertyTree.")),
        }
    }

    fn read_string(&mut self) -> Result<String> {
        let scenario_len = self.read_u8()?;
        let mut scenario_name = vec![0; scenario_len as usize];
        self.read_exact(&mut scenario_name)?;

        Ok(String::from_utf8_lossy(&scenario_name).to_string())
    }

    // Strings in PropertyTrees have an extra byte to inform us if they are empty
    fn read_pt_string(&mut self) -> Result<Option<String>> {
        if self.read_bool()? {
            Ok(None)
        } else {
            self.read_string().map(Some)
        }
    }

    fn read_u16_optimized(&mut self) -> Result<u16> {
        let mut version: u16 = self.read_u8()?.into();
        if version == 255 {
            version = self.read_u16::<LittleEndian>()?;
        }
        Ok(version)
    }

    fn read_mod(&mut self) -> Result<ModIdent> {
        let mod_name = self.read_string()?;

        let version_major = self.read_u16_optimized()?;
        let version_minor = self.read_u16_optimized()?;
        let version_patch = self.read_u16_optimized()?;

        // We don't care about the CRC
        let _crc = self.read_u32::<LittleEndian>()?;

        Ok(ModIdent {
            name: mod_name,
            version_req: Some(VersionReq::exact(&Version::new(
                version_major as u64,
                version_minor as u64,
                version_patch as u64,
            ))),
        })
    }
}

/// All types that implement `Read` get methods defined in `ReadBytesFmm`
/// for free.
impl<R: io::Read + ?Sized> ReadFactorioDat for R {}
