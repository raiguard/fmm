use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs;
use std::io::{Cursor, Seek, SeekFrom};
use std::path::PathBuf;

use crate::read::PropertyTree;

pub struct ModSettings {
    pub settings: PropertyTree,

    path: PathBuf,
    version_major: u16,
    version_minor: u16,
    version_patch: u16,
    version_build: u16,
}

impl ModSettings {
    pub fn new(mut path: PathBuf) -> Result<Self> {
        path.push("mod-settings.dat");
        let mut cursor = Cursor::new(fs::read(&path)?);

        let version_major = cursor.read_u16::<LittleEndian>()?;
        let version_minor = cursor.read_u16::<LittleEndian>()?;
        let version_patch = cursor.read_u16::<LittleEndian>()?;
        let version_build = cursor.read_u16::<LittleEndian>()?;

        // This is a one-byte flag that is always false
        cursor.seek(SeekFrom::Current(1))?;

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
        let contents = vec![
            self.version_major,
            self.version_minor,
            self.version_patch,
            self.version_build,
            0,
        ];
        // contents.append(self.settings.write());
        // let mut file = fs::write(self.path, &contents);

        unimplemented!()
    }
}
