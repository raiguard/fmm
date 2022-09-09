# fmm

`fmm` is a CLI mod manager for Factorio. Easily enable, disable, download,
remove, update, upload, or sync mods with a save file. Dependencies will be
automatically downloaded and enabled.

## Installation

Pre-built binaries for Linux, macOS, and Windows will be available as soon as
I get around to migrating to Sourcehut Builds.

AUR packages for compiling and downloading the latest binary will be available
in the future as well.

### Building from source

Requires [Rust](https://rust-lang.org).

```
git clone https://git.sr.ht/~raiguard/fmm & cd fmm
cargo install --locked --force --path .
```

This will install the `fmm` binary to your `$PATH`.

### Building from crates.io

Run the following command:

```
cargo install fmm
```

This will install the `fmm` binary to your `$PATH`.

## Usage

```
# Enable Space Exploration and all dependencies
fmm e space-exploration
# Enable a user-defined mod set
fmm es MyModSet
# Search the mod portal
fmm l "logistic train network"
```

See `fmm --help` for all commands.

## Configuration

`fmm` accepts a `--config` flag with a path to a [`toml`](https://toml.io/en/)
configuration file. If `--config` is not provided, `fmm` will look for this
file in the following location:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| Linux    | /home/rai/.config/fmm/fmm.toml                      |
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.toml |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.toml           |

Values passed as flags will override those in the config file.

[EXAMPLE CONFIGURATION](./fmm.toml)
