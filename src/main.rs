use semver::Version;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

mod types;

use types::*;

#[derive(StructOpt)]
#[structopt(name = "fmm", about = "Manage your Factorio mods.")]
struct App {
    #[structopt(long)]
    dir: PathBuf,
    #[structopt(short, long)]
    disable: Vec<InputMod>,
    #[structopt(short, long)]
    enable: Vec<InputMod>,
}

// DESIGN NOTES:
// - Get a list of all mods + versions in the folder _without_ reading the ZIP files (use filenames)
// - Only read ZIPs if we need to get dependencies or other info
// - Cache will only be used once we have advanced features that would benefit from it

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::from_args();

    // Get all mods in the directory
    let directory_mods = fs::read_dir(&app.dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();

            // TODO: Folders can be versionless, in which case we have to parse their info.json
            let (mod_name, version) = file_name.to_str()?.rsplit_once("_")?;
            let (version, _) = version.rsplit_once(".").unwrap_or((version, "")); // Strip file extension

            Some((mod_name.to_string(), Version::parse(version).ok()?))
        })
        .fold(HashMap::new(), |mut directory_mods, (mod_name, version)| {
            let versions = directory_mods.entry(mod_name).or_insert_with(Vec::new);

            let index = versions
                .binary_search(&version)
                .unwrap_or_else(|index| index);
            versions.insert(index, version);

            directory_mods
        });

    // Parse mod-list.json
    let mut mlj_path = app.dir;
    mlj_path.push("mod-list.json");
    let enabled_versions = std::fs::read_to_string(&mlj_path)?;
    let mut mod_list_json: ModListJson = serde_json::from_str(&enabled_versions)?;

    // Enable specified mods
    for mod_data in app.enable {
        if directory_mods.contains_key(&mod_data.name) {
            let mod_state = mod_list_json
                .mods
                .iter_mut()
                .find(|mod_state| mod_data.name == mod_state.name);

            println!("Enabled {}", &mod_data);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = true;
                mod_state.version = mod_data.version;
            } else {
                mod_list_json.mods.push(ModListJsonMod {
                    name: mod_data.name.to_string(),
                    enabled: true,
                    version: mod_data.version,
                });
            }
        } else {
            println!("Could not find {}", &mod_data);
        }
    }

    // Disable specified mods
    for mod_data in app.disable {
        if directory_mods.contains_key(&mod_data.name) {
            let mod_state = mod_list_json
                .mods
                .iter_mut()
                .find(|mod_state| mod_data.name == mod_state.name);

            println!("Disabled {}", &mod_data);

            if let Some(mod_state) = mod_state {
                mod_state.enabled = false;
                mod_state.version = None;
            }
        } else {
            println!("Could not find {}", &mod_data);
        }
    }

    fs::write(&mlj_path, serde_json::to_string_pretty(&mod_list_json)?)?;

    Ok(())
}
