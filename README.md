# fmm

A CLI mod manager for Factorio. Easily download and enable mods, sync with save
files or log file checksums, and upload mod zip files to the portal.

## Usage

```
fmm <operation> [flags...] [args...]
flags:
	-x                  Read args from stdin (one per line)
operations:
	disable [args...]   Disable the given mods, or all mods if none are given
	enable  [args...]   Enable the given mods and their dependencies, downloading if necessary
	help                Show usage information
	list    [files...]  List all mods in the mods directory, or in the given save files
	sync    [args...]   Disable all mods, then download and enable the given mods
	upload  [files...]  Upload the given mod zip files to the mod portal
```

Mods are specified by `name` or `name_version`.

## Configuration

fmm will check the current directory and the previous directory for a Factorio
installation. If neither is valid, it will fall back to the directory specified
by the `FACTORIO_PATH` environment variable.

For uploading mods, specify your API key with the `FACTORIO_API_KEY` variable.

If you have logged in to your Factorio account, fmm will automatically pull
your username and token from the `player-data.json` file. Alternatively, you
can specify them with `FACTORIO_USERNAME` and `FACTORIO_TOKEN` respectively.

## Installation

Distribution packages will be available once the project is stable.

## Building

Dependencies:
- [go](https://go.dev)
- [scdoc](https://git.sr.ht/~sircmpwn/scdoc) (for man pages)

```
make
sudo make install
```

If `make` is unavailable, `go build` should work just fine.
