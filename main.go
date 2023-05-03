package main

import (
	"fmt"
	"os"

	"github.com/adrg/xdg"
)

var (
	apiKey           string = ""
	configPath       string = "./fmm.ini"
	downloadToken    string = ""
	downloadUsername string = ""
	modsDir          string = "."
)

const usageStr string = `usage: fmm <operation> [args...]
operations:
	disable [mods...]   Disable the given mods, or all mods if none are given
	enable  [mods...]   Enable the given mods and their dependencies
	help                Show usage information
	upload  [files...]  Upload the given mod zip files to the mod portal`

func printUsage(msg ...any) {
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
	xdgConfigPath, err := xdg.ConfigFile("fmm/fmm.ini")
	if err == nil {
		configPath = xdgConfigPath
	}
	if err := parseConfig(configPath); err != nil {
		abort("could not parse config file:", err)
	}

	if _, err := os.Stat("mod-list.json"); err == nil {
		fmt.Println("Using current directory")
		modsDir = "."
	}

	args := os.Args[1:]
	if len(args) == 0 {
		abort("no operation was specified")
	}

	var task func([]string)
	switch args[0] {
	case "disable", "d":
		task = disable
	case "enable", "e":
		task = enable
	case "help", "h", "-h", "--help":
		printUsage()
	// case "install", "i":
	// 	task = install
	// case "sync", "s":
	// 	task = sync
	case "upload", "ul":
		task = upload
	default:
		abort(fmt.Sprintf("unrecognized operation %s", args[0]))
	}
	task(args[1:])
}
