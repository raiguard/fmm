use crate::mod_ident::ModIdent;
use crate::version::Version;
use anyhow::anyhow;
use anyhow::Result;
use byteorder::WriteBytesExt;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::io::SeekFrom;

pub type DatReader = Cursor<Vec<u8>>;

#[derive(Clone, Debug)]
pub enum PropertyTree {
    None,
    Boolean(bool),
    Number(f64),
    String(Option<String>),
    List(Vec<PropertyTree>),
    Dictionary(HashMap<String, PropertyTree>),
}

impl PropertyTree {
    /// Load a PropertyTree from binary data.
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

    /// Index into a PropertyTree list or dictionary.
    #[allow(unused)]
    pub fn get<T: PropertyTreeKey>(&self, key: T) -> Option<&Self> {
        key.index_into(self)
    }

    /// Mutably index into a PropertyTree list or dictionary.
    #[allow(unused)]
    pub fn get_mut<T: PropertyTreeKey>(&mut self, key: T) -> Option<&mut Self> {
        key.index_into_mut(self)
    }

    /// Returns `true` if the `PropertyTree` is a list.
    #[allow(unused)]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// If the `PropertyTree` is a list, returns the associated vector. Otherwise returns `None`.
    #[allow(unused)]
    pub fn as_list(&self) -> Option<&Vec<PropertyTree>> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// If the `PropertyTree` is a list, returns the associated mutable vector. Otherwise returns `None`.
    #[allow(unused)]
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<PropertyTree>> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Returns `true` if the `PropertyTree` is a list.
    #[allow(unused)]
    pub fn is_dictionary(&self) -> bool {
        matches!(self, Self::Dictionary(_))
    }

    /// If the `PropertyTree` is a dictionary, returns the associated hashmap. Otherwise returns `None`.
    pub fn as_dictionary(&self) -> Option<&HashMap<String, PropertyTree>> {
        match self {
            Self::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }

    /// If the `PropertyTree` is a dictionary, returns the associated mutable hashmap. Otherwise returns `None`.
    pub fn as_dictionary_mut(&mut self) -> Option<&mut HashMap<String, PropertyTree>> {
        match self {
            Self::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }

    /// Serializes the `PropertyTree` into the given bytestream.
    pub fn write(&self, output: &mut Vec<u8>) -> Result<()> {
        // Each PropertyTree type has a flag as the second byte that doesn't matter, so we just ignore it
        match self {
            PropertyTree::None => {
                // PropertyTree type
                output.write_u8(0)?;
                // Internal flag
                output.write_u8(0)?;
            }
            PropertyTree::Boolean(bool) => {
                // PropertyTree type
                output.write_u8(1)?;
                // Internal flag
                output.write_u8(0)?;
                // Data
                output.write_u8(if *bool { 1 } else { 0 })?;
            }
            PropertyTree::Number(num) => {
                // PropertyTree type
                output.write_u8(2)?;
                // Internal flag
                output.write_u8(0)?;
                // Data
                output.write_f64::<LittleEndian>(*num)?;
            }
            PropertyTree::String(str) => {
                // PropertyTree type
                output.write_u8(3)?;
                // Internal flag
                output.write_u8(0)?;
                // Data
                output.write_pt_string(str)?;
            }
            PropertyTree::List(list) => {
                // PropertyTree type
                output.write_u8(4)?;
                // Internal flag
                output.write_u8(0)?;

                // Length of the list
                output.write_u32::<LittleEndian>(list.len() as u32)?;

                // List contents
                for item in list {
                    // List keys are empty strings
                    output.write_pt_string(&None)?;
                    item.write(output)?;
                }
            }
            PropertyTree::Dictionary(dict) => {
                // PropertyTree type
                output.write_u8(5)?;
                // Internal flag
                output.write_u8(0)?;

                // Length of the list
                output.write_u32::<LittleEndian>(dict.len() as u32)?;

                // Dictionary contents
                for (key, value) in dict {
                    // Dictionary keys always exist
                    output.write_pt_string(&Some(key.to_string()))?;
                    value.write(output)?;
                }
            }
        };

        Ok(())
    }
}

pub trait PropertyTreeKey {
    fn index_into<'a>(&self, pt: &'a PropertyTree) -> Option<&'a PropertyTree>;

    fn index_into_mut<'a>(&self, pt: &'a mut PropertyTree) -> Option<&'a mut PropertyTree>;
}

impl PropertyTreeKey for &str {
    fn index_into<'a>(&self, pt: &'a PropertyTree) -> Option<&'a PropertyTree> {
        match pt {
            PropertyTree::Dictionary(dict) => dict.get(*self),
            _ => None,
        }
    }

    fn index_into_mut<'a>(&self, pt: &'a mut PropertyTree) -> Option<&'a mut PropertyTree> {
        match pt {
            PropertyTree::Dictionary(dict) => dict.get_mut(*self),
            _ => None,
        }
    }
}

impl PropertyTreeKey for usize {
    fn index_into<'a>(&self, pt: &'a PropertyTree) -> Option<&'a PropertyTree> {
        match pt {
            PropertyTree::List(list) => list.get(*self),
            _ => None,
        }
    }

    fn index_into_mut<'a>(&self, pt: &'a mut PropertyTree) -> Option<&'a mut PropertyTree> {
        match pt {
            PropertyTree::List(list) => list.get_mut(*self),
            _ => None,
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

    fn read_mod(&mut self) -> Result<ModIdent> {
        let mod_name = self.read_string()?;

        let version_major = self.read_u16_optimized()?;
        let version_minor = self.read_u16_optimized()?;
        let version_patch = self.read_u16_optimized()?;

        // We don't care about the CRC
        let _crc = self.read_u32::<LittleEndian>()?;

        Ok(ModIdent {
            name: mod_name,
            version: Some(Version::new(
                version_major as u32,
                version_minor as u32,
                version_patch as u32,
                None,
            )),
        })
    }

    fn read_string(&mut self) -> Result<String> {
        let string_len = self.read_u32_optimized()?;
        let mut scenario_name = vec![0; string_len as usize];
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
        let mut num: u16 = self.read_u8()?.into();
        if num == 255 {
            num = self.read_u16::<LittleEndian>()?;
        }
        Ok(num)
    }

    fn read_u32_optimized(&mut self) -> Result<u32> {
        let mut num: u32 = self.read_u8()?.into();
        if num == 255 {
            num = self.read_u32::<LittleEndian>()?;
        }
        Ok(num)
    }
}

/// All types that implement `Read` get methods defined in `ReadFactorioDat` for free.
impl<R: io::Read + ?Sized> ReadFactorioDat for R {}

pub trait WriteFactorioDat: io::Write {
    fn write_u32_optimized(&mut self, num: u32) -> io::Result<()> {
        if num < 255 {
            self.write_u8(num as u8)?;
        } else {
            // Represented as the first byte being 255
            self.write_u8(255)?;
            self.write_u32::<LittleEndian>(num)?;
        }

        Ok(())
    }

    fn write_pt_string(&mut self, str: &Option<String>) -> io::Result<()> {
        if let Some(str) = str {
            // Not-empty flag
            self.write_u8(0)?;
            // String length
            self.write_u32_optimized(str.len() as u32)?;
            // String contents
            self.write_all(str.as_bytes())?;
        } else {
            // Empty flag
            self.write_u8(1)?;
        }

        Ok(())
    }
}

/// All types that implement `Write` get methods defined in `WriteFactorioDat` for free.
impl<R: io::Write + ?Sized> WriteFactorioDat for R {}
