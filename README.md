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
usage: fmm <options> <subcommand>
environment variables:
    FMM_TOKEN          oauth token for the mod portal
options:
    --all              when using the clean subcommand, remove all non-symlink mods
    --config <PATH>    path to a custom configuration file
    --force -f         always download, overwriting an existing mod if necessary
    --game-dir <PATH>  path to the game directory
    --mods-dir <PATH>  path to the mods directory
    --nodisable -d     when using sync subcommands, keep current mods enabled
    --token <TOKEN>    oauth token for the mod portal
subcommands:
    {clean, c}                remove out-of-date mod versions, leaving only the newest version; ignores symlinked mods
    {disable, d}     <MODS>   disable the given mods, or all mods if no mods are given
    {download, dl}   <MODS>   download the given mods
    {enable, e}      <MODS>   enable the given mods
    {enable-set, es} <SET>    enable the mods from the given mod set
    {query, q}       <MODS>   query the local mods folder
    {remove, r}      <MODS>   remove the given mods
    {search, l}      <QUERY>  search for mods on the mod portal
    {sync, s}        <MODS>   enable the given mods, downloading if necessary, and disable all other mods
    {sync-file, sf}  <PATH>   enable the mods from the given save file, downloading if necessary, disable all other mods, and sync mod startup settings
    {sync-list, sl}  <PATH>   enable the mods from the given mod-list.json, downloading if necessary, and disable all other mods
    {sync-set, ss}   <SET>    enable the mods from the given mod set, downloading if necessary, and disable all other mods
    {update, u}      <MODS>   update the given mods, or all mods if no mods are given
    {upload, ul}     <PATH>   upload the given mod zip file to the mod portal
```

TODO: Add a man page with more details

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
