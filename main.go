package main

import (
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path"
	"strings"

	"github.com/adrg/xdg"
)

var (
	apiKey           string = ""
	configPath       string = "./fmm.ini"
	downloadToken    string = ""
	downloadUsername string = ""
	gameDir          string = "."
	modsDir          string = ""
)

const usageStr string = `usage: fmm <operation> [flags...] [args...]
flags:
	-x                  Read args from stdin (one per line)
operations:
	disable [args...]   Disable the given mods, or all mods if none are given
	enable  [args...]   Enable the given mods and their dependencies, downloading if necessary
	help                Show usage information
	list    [files...]   List all mods in the mods directory, or in the given save files
	sync    [args...]   Disable all mods, then download and enable the given mods
	upload  [files...]  Upload the given mod zip files to the mod portal`

func printUsage(msg ...any) {
	if len(msg) > 0 {
		errorln(msg...)
	}
	errorln(usageStr)
	os.Exit(1)
}

// CONTROL FLOW:
// - Read config file
// - Parse input list into list of ModIdent, taking from:
//   - User representation ("EditorExtensions")
//   - Save file
//   - Log file
//   - Mod-list.json
//   - Mod sets?
// - Add missing dependencies, fetching from portal if needed
//   - Keep note of which are present and which need to be downloaded
// - Check for incompatibilities and circular dependencies in list and currently enabled mods
// - Confirm actions with user?
// - Execute each enable or download action

func main() {
	args := os.Args[1:]
	if len(args) == 0 {
		printUsage("no operation was specified")
	}

	var task func([]string)
	switch args[0] {
	case "disable", "d":
		task = disable
	case "enable", "e":
		task = enable
	case "help", "h", "-h", "--help":
		printUsage()
	case "list", "ls":
		task = list
	case "sync", "s":
		task = sync
	case "upload", "ul":
		task = upload
	default:
		printUsage("unrecognized operation", args[0])
	}

	if xdgConfigPath, err := xdg.SearchConfigFile("fmm/fmm.ini"); err == nil {
		configPath = xdgConfigPath
	}
	if err := parseConfig(configPath); err != nil && !errors.Is(err, fs.ErrNotExist) {
		abort("could not parse config file:", err)
	}

	if isFactorioDir(".") {
		fmt.Println("Using current directory")
		gameDir = "."
	} else if isFactorioDir("..") {
		fmt.Println("Using current directory")
		gameDir = ".."
	}
	modsDir = path.Join(gameDir, "mods")

	// TODO: Auto-create mods folder and mod-list.json
	if !entryExists(modsDir, "mod-list.json") {
		abort("mods directory does not contain info.json")
	}

	// Read from stdin if '-x' was provided
	args = args[1:]
	if len(args) > 0 && args[0] == "-x" {
		bytes, _ := io.ReadAll(os.Stdin)
		if len(bytes) == 0 {
			// Nothing was provided
			return
		}
		args = strings.Split(strings.TrimSpace(string(bytes)), "\n")
	}

	task(args)
}

func isFactorioDir(dir string) bool {
	if !entryExists(dir, "data", "changelog.txt") {
		return false
	}
	if !entryExists(dir, "data", "base", "info.json") {
		return false
	}
	return entryExists(dir, "config-path.ini") || entryExists(dir, "config", "config.ini")
}

func entryExists(pathParts ...string) bool {
	_, err := os.Stat(path.Join(pathParts...))
	return err == nil
}
