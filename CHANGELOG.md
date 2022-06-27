# [0.7.0] - ??????????

## Added
- `update` and `upload` subcommands

## Changed
- Restructured existing functionality into distinct and easy-to-understand subcommands
- `query` subcommand now searches the local mod directory, while `search` searches the mod portal
- `sync-file` will sync the exact mods in the file, unless `sync_latest_versions` is enabled, in which case it will scan for dependencies

## Removed
- `auto_download` and `sync_startup_settings` settings

# [0.6.0] - 2022-04-23

## Added
- `query` subcommand, which will search the mod portal for matching mods

## Changed
- All mod dependencies are resolved before download or enabling begins
- Many errors that previously caused the program to abort are now handled gracefully
- Restructured existing commands and options to fit under a new `sync` subcommand

# [0.5.0] - 2022-03-01

## Added
- Colors to command output
- Mod downloading - mods that you do not have will automatically be downloaded when enabling or syncing
  - This can be disabled in the config file
- Startup settings syncing with `--sync`
- Section to config file for mod portal username and token
- `--game-dir` flag and option, used for downloading mods

## Changed
- `--sync` will sync the exact mod version instead of the latest version by default
- Renamed `--dir` flag to `--mods-dir` and `directory` option to `mods_dir`

## Fixed
- `--sync` would completely break if a mod had a version number greater than 255

# [0.4.0] - 2021-11-13

## Added
- `--list` flag, for listing all of the mods in the directory
- `--sync` flag, for enabling mods that are in the given save file

# [0.3.0] - 2021-11-06

## Added
- `--enable-set` flag, for enabling pre-defined sets of mods
  - These sets can be configured in `fmm.toml`

## Changed
- "Mod is already enabled" messages were removed - they hurt more than they helped

## Fixed
- Fixed that versionless mod folders with an underscore would not be parsed correctly

# [0.2.0] - 2021-11-03

## Added
- `--remove` flag

## Fixed
- Fixed GitHub release workflow

# [0.1.0] - 2021-11-03

## Added
- Initial release
