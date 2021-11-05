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
fmm --enable space-exploration
```

See `fmm --help` for all subcommands.

## Features

- Enable mods and their dependencies
- Disable mods
- Enable or disable all mods at once
- Enable pre-defined sets of mods
- Set your default directory by using a config file
- Remove mods from your mods directory

## Configuration

`fmm` accepts a `--config` flag with a path to a [`toml`](https://toml.io/en/) configuration file. If `--config` is not provided, `fmm` will look for this file in `$XDG_CONFIG_HOME/fmm/fmm.toml` and source it if it exists. Here is the format of this file:

```toml
# The path to the Factorio mods directory
directory = "/home/rai/.factorio/mods/"

# Customizable mod sets
# Each key will be available for use in the `--enable-set` flag
# Dependencies will automatically be enabled
# Specific versions may be specified by appending `@version` to the mod name
[sets]
EditorExtensions = ["EditorExtensions", "Sandbox"]
Krastorio2Beta = ["Krastorio2@1.2.0", "EditorExtensions", "Sandbox"]
```

## Roadmap

Roughly in this order:

- Mod sets
- Sync with `mod-list.json`
- Sync with save
- Automatically publish to AUR
- Create new mod
- Package mod
- Datestamp and increment mod version
- Download mods
- Upload mods
