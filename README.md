# Factorio Mod Manager

`fmm` is a basic CLI Factorio mod manager. Is is completely portable and runs on all major platforms.

## Getting started

## Installing

Download the binary for your system from the [releases](https://github.com/raiguard/fmm/releases) page and place it on your `PATH`.

Coming soon: an AUR package.

## Building

Requires [Rust](https://rust-lang.org) nightly.

```
git clone https://github.com/raiguard/fmm & cd fmm
cargo build
```

### Installing to `PATH`

You can build `fmm` from source and install it on your `PATH` like this:

```
cargo install --locked --force --path .
```

Or just run this without cloning the repository:

```
cargo install fmm
```

## Usage

```
# Enable Space Exploration and all dependencies
fmm -Se space-exploration
# Enable a user-defined mod set
fmm -Ss MyModSet
# Search the mod portal
fmm -Q "logistic train network"
```

See `fmm --help` for all commands.

## Features

- List all mods in the mods directory
- Enable mods and their dependencies
- Disable mods
- Enable or disable all mods at once
- Enable pre-defined sets of mods
- Sync enabled mods with a save file
- Download mods from the portal
- Remove mods from your mods directory

## Configuration

`fmm` accepts a `--config` flag with a path to a [`toml`](https://toml.io/en/) configuration file. If `--config` is not provided, `fmm` will look for this file in the following location:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| Linux    | /home/rai/.config/fmm/fmm.toml                      |
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.toml |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.toml           |

Values passed as flags will override those in the config file.

[EXAMPLE CONFIGURATION](./fmm.toml)

## Roadmap

Roughly in this order:

- Sync with `mod-list.json`
- Local config
- Automatically publish to AUR
- Create new mod
- Package mod
- Datestamp and increment mod version
- Upload mods
