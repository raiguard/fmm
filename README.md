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

You can build `fmm` and install it on your `PATH` like this:

```
cargo install --locked --forced --path .
```

## Features

- Enable mods and their dependencies
- Disable mods
- Enable or disable all mods at once
- Set your default directory by using a config file
- Remove mods from your mods directory

## Configuration

`fmm` is very bare-bones, but does support a [`toml`](https://toml.io/en/) file for setting the default directory. You can set the path to this file with the `--config` flag, or place it in `$XDG_CONFIG_HOME/fmm/fmm.toml` for it to be sourced automatically.

```toml
directory = "/home/rai/.factorio/mods"
```

## Roadmap

Roughly in this order:

- Automatically publish to AUR
- Mod sets
- Sync with `mod-list.json`
- Sync with log file?
- Sync with save
- Create new mod
- Package mod
- Datestamp and increment mod version
- Download mods
- Upload mods
