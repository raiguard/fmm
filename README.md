# fmm

A CLI mod manager for Factorio. Easily enable, disable, download, remove,
update, upload, or sync mods with a save file. Dependencies will be
automatically downloaded and enabled.

## Status

This project is currently being rewritten from the ground up in Go. Most
features are missing, and it has bugs!

## Usage

```
fmm enable space-exploration
fmm download Krastorio2@1.2.0
fmm sync-file ~/downloads/cool-save-file.zip
```

Read the [man pages](./man) to learn more.

## Installation

Distribution packages will be available once the project is usable.

## Building

Dependencies:
- [go](https://go.dev)
- [scdoc](https://git.sr.ht/~sircmpwn/scdoc) (for man pages)

```
make
sudo make install
```

### Windows and macOS

fmm only officially supports Linux. However, if you clone the repository and
build with `go build`, it should work on other platforms as well. There is no
Linux-specific code in the codebase.

Configuration file locations:

| Platform | Path                                                |
| -------- | --------------------------------------------------- |
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.toml |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.toml           |

## Contributing

Please send bug reports, questions, or patches to the
[mailing list](https://lists.sr.ht/~raiguard/public-inbox).
