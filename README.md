# fmm

A CLI mod manager for Factorio. Easily download and enable mods, sync with save
files or log file checksums, and upload mod zip files to the portal.

## Usage

```
fmm help
fmm enable RecipeBook
fmm sync ~/downloads/save-file.zip
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
| macOS    | /Users/Rai/Library/Application Support/fmm/fmm.ini  |
| Windows  | C:\Users\Rai\AppData\Roaming\fmm\fmm.ini            |
