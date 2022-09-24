# fmm

A CLI mod manager for Factorio. Easily enable, disable, download, remove,
update, upload, or sync mods with a save file. Dependencies will be
automatically downloaded and enabled.

## Usage

```
fmm enable space-exploration
fmm download Krastorio2
fmm sync-file ~/downloads/cool-save-file.zip
```

Read the [man pages](./man) to learn more.

## Installation

AUR packages for compiling from source and for downloading the latest binary
will be available soon.

## Building

Dependencies:
- [rust](https://rust-lang.org) (cargo)
- [scdoc](https://git.sr.ht/~sircmpwn/scdoc) (for man pages)

```
make release
sudo make install
```

### Windows and macOS

fmm only officially supports Linux. However, if you clone the repository and
build with `cargo`, it should work on other platforms as well. There is no
Linux-specific code in the codebase.

Configuration file locations:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.toml |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.toml           |

## Contributing

Please send bug reports, questions, or patches to the
[mailing list](https://lists.sr.ht/~raiguard/public-inbox).
