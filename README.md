# ⚠️ This project has migrated to [Codeberg](https://codeberg.org/raiguard/fmm).

# fmm

A CLI mod manager for Factorio. Easily download and enable mods, sync with save
files or log file checksums, and upload mod zip files to the portal.

## Installation

Grab the latest release from the [releases page](https://github.com/raiguard/fmm/releases).

Distribution packages will be available once the project is stable.

## Building

**Requirements:**
- [Go](https://go.dev) 1.21 or newer

```
make
```

Run tests:

```
make test
```

## Usage

```
usage: fmm <command> [args...]
commands:
  add     [args...]   Download and enable the given mods and their dependencies.
  disable [args...]   Disable the given mods, or all mods if none are given.
  enable  [args...]   Enable the given mods and their dependencies.
  help                Show usage information.
  list    [files...]  List all mods in the mods directory, or in the given save files.
  sync    [args...]   Disable all mods, then download and enable the given mods and their dependencies.
                      If a save file is provided, merge startup mod settings with the settings contained in that save.
  update  [args...]   Update the given mods, or all mods if none are given.
  upload  [files...]  Upload the given mod zip files to the mod portal.
```

Mods are specified by `name` or `name_version`.

## Configuration

fmm will check the current directory and the previous directory for a Factorio
installation. If neither is valid, it will fall back to the directory specified
by the `FACTORIO_PATH` environment variable.

If the mods directory is not contained within the game data directory (i.e. you
installed via Steam), specify the `FACTORIO_MODS_PATH` environment variable to
point fmm to the correct mods directory.

For uploading mods, specify your API key with the `FACTORIO_API_KEY` variable.

If you have logged in to your Factorio account, fmm will automatically pull
your username and token from the `player-data.json` file. Alternatively, you
can specify them with `FACTORIO_USERNAME` and `FACTORIO_TOKEN` respectively.

## TODO

Not necessarily in order.

- Man pages
- Mod creation and packaging
- Automated testing CI
