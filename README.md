# fmm

`fmm` is a CLI mod manager for Factorio. Easily enable, disable, download,
remove, update, upload, or sync mods with a save file. Dependencies will be
automatically downloaded and enabled.

Please send bug reports, questions, or patches to the
[mailing list](https://lists.sr.ht/~raiguard/public-inbox).

## Installation

fmm will soon have pre-built Linux binaries available on the
[refs](https://git.sr.ht/~raiguard/fmm/refs) tab.

AUR packages for compiling from source and for downloading the latest binary
will be available soon.

### Building from source

Requires [Rust](https://rust-lang.org).

```
git clone https://git.sr.ht/~raiguard/fmm & cd fmm
cargo install --locked --force --path .
```

This will install the `fmm` binary to your `$PATH`.

## Usage

```
fmm enable space-exploration
fmm download Krastorio2
fmm sync-file ~/downloads/cool-save-file.zip
```

See [**fmm(1)**](./fmm.1.scd).

## Windows and macOS

fmm primarily targets Linux, but it should work just fine on Windows and macOS
as well. Pre-built binaries for these platforms are not provided, but the
steps to build from source should be the same.

Configuration file locations:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.toml |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.toml           |
