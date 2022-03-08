use crate::dat::PropertyTree;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

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
    pub fn new(path: &Path) -> Result<Self> {
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

    /// Merge the stored startup settings with the passed settings
    /// Settings that are stored will be overwritten with the passed settings
    pub fn merge_startup_settings(&mut self, other: &PropertyTree) -> Result<()> {
        let startup_settings = self
            .settings
            .get_mut("startup")
            .ok_or_else(|| anyhow!("No startup settings in mod-settings.dat"))?
            .as_dictionary_mut()
            .ok_or_else(|| anyhow!("Could not read PropertyTree dictionary."))?;

        for (setting_name, setting_value) in other
            .as_dictionary()
            .ok_or_else(|| anyhow!("Could not read PropertyTree dictionary"))?
        {
            startup_settings.insert(setting_name.clone(), setting_value.clone());
        }

        Ok(())
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
