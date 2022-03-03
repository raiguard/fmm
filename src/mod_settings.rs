use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use crate::dat::PropertyTree;

#[derive(Debug)]
pub struct ModSettings {
    pub settings: PropertyTree,

    path: PathBuf,
    version_major: u16,
    version_minor: u16,
    version_patch: u16,
    version_build: u16,
}

impl ModSettings {
    pub fn new(mut path: &PathBuf) -> Result<Self> {
        let path = path.join("mod-settings.dat");
        let mut cursor = Cursor::new(fs::read(&path)?);

        let version_major = cursor.read_u16::<LittleEndian>()?;
        let version_minor = cursor.read_u16::<LittleEndian>()?;
        let version_patch = cursor.read_u16::<LittleEndian>()?;
        let version_build = cursor.read_u16::<LittleEndian>()?;

        // This is a one-byte flag that is always false
        cursor.read_u8()?;

        let mod_settings = PropertyTree::load(&mut cursor)?;

        Ok(Self {
            settings: mod_settings,
            path,
            version_major,
            version_minor,
            version_patch,
            version_build,
        })
    }

    pub fn write(&self) -> Result<()> {
        let mut output = vec![];

        // Factorio version number
        output.write_u16::<LittleEndian>(self.version_major)?;
        output.write_u16::<LittleEndian>(self.version_minor)?;
        output.write_u16::<LittleEndian>(self.version_patch)?;
        output.write_u16::<LittleEndian>(self.version_build)?;
        // Internal flag - always false
        output.push(false as u8);

        // Settings PropertyTree
        self.settings.write(&mut output)?;

        fs::write(&self.path, output)?;

        Ok(())
    }
}
