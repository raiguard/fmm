package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path"
	"strings"
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

	if isFactorioDir(".") {
		fmt.Println("Using current directory")
		gameDir = "."
	} else if isFactorioDir("..") {
		fmt.Println("Using previous directory")
		gameDir = ".."
	} else {
		gameDir = os.Getenv("FACTORIO_PATH")
		if !isFactorioDir(gameDir) {
			abort("invalid game directory")
		}
	}
	modsDir = path.Join(gameDir, "mods")
	// TODO: Auto-create mods folder and mod-list.json
	if !entryExists(modsDir, "mod-list.json") {
		abort("mods directory does not contain info.json")
	}

	apiKey = os.Getenv("FACTORIO_API_KEY")
	err := getPlayerData()
	if err != nil {
		abort(err)
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

func getPlayerData() error {
	downloadUsername = os.Getenv("FACTORIO_USERNAME")
	downloadToken = os.Getenv("FACTORIO_TOKEN")

	playerDataJsonPath := path.Join(gameDir, "player-data.json")
	if !entryExists(playerDataJsonPath) {
		return nil
	}

	data, err := os.ReadFile(playerDataJsonPath)
	if err != nil {
		return errors.New("Unable to read player-data.json")
	}
	var playerDataJson PlayerDataJson
	err = json.Unmarshal(data, &playerDataJson)
	if err != nil {
		return errors.New("Invalid player-data.json format")
	}
	if playerDataJson.ServiceToken != nil {
		downloadToken = *playerDataJson.ServiceToken
	}
	if playerDataJson.ServiceUsername != nil {
		downloadUsername = *playerDataJson.ServiceUsername
	}

	return nil
}
