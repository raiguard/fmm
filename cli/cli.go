package cli

import (
	"errors"
	"fmt"
	"io"
	"os"
	"strings"

	fmm "github.com/raiguard/fmm/lib"
)

const usageStr string = `usage: fmm <command> [flags...] [args...]
flags:
	-x                  Read args from stdin (one per line)
commands:
	disable [args...]   Disable the given mods, or all mods if none are given
	enable  [args...]   Enable the given mods and their dependencies, downloading if necessary
	help                Show usage information
	list    [files...]  List all mods in the mods directory, or in the given save files
	sync    [args...]   Disable all mods, then download and enable the given mods
	upload  [files...]  Upload the given mod zip files to the mod portal`

func Run() {
	args := os.Args[1:]
	if len(args) == 0 {
		printUsage("no operation was specified")
	}

	var task func(*fmm.Manager, []string)
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

	manager, err := fmm.NewManager(".")
	if err != nil {
		manager, err = fmm.NewManager(os.Getenv("FACTORIO_PATH"))
		if err != nil {
			abort(err)
		}
	}

	if !manager.HasPlayerData() {
		manager.SetPlayerData(fmm.PlayerData{
			Token:    os.Getenv("FACTORIO_TOKEN"),
			Username: os.Getenv("FACTORIO_USERNAME"),
		})
	}

	manager.SetApiKey(os.Getenv("FACTORIO_API_KEY"))

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

	task(manager, args)

	if err := manager.Save(); err != nil {
		errorln(errors.Join(errors.New("unable to save modifications"), err))
	}
}

func disable(manager *fmm.Manager, args []string) {
	if len(args) == 0 {
		manager.DisableAll()
		fmt.Println("disabled all mods")
		return
	}

	mods := parseCliInput(args, false)
	for _, mod := range mods {
		if err := manager.Disable(mod.Name); err != nil {
			errorf("failed to disable %s\n", mod.ToString())
			errorln(err)
		} else {
			fmt.Println("disabled", mod.Name)
		}
	}
}

func enable(manager *fmm.Manager, args []string) {
	mods := parseCliInput(args, true)

	for _, mod := range mods {
		// if !mod.IsPresent {
		// 	err := portalDownloadMod(Dependency{mod.Ident, DependencyRequired, VersionEq})
		// 	if err != nil {
		// 		errorln(err)
		// 		continue
		// 	}
		// }
		if err := manager.Enable(mod.Name, mod.Version); err != nil {
			errorf("failed to enable %s\n", mod.ToString())
			errorln(err)
		} else {
			fmt.Println("enabled", mod.ToString())
		}
	}
}

func list(manager *fmm.Manager, args []string) {
	// if len(args) == 0 {
	// 	dir := newDir(manager.modsDir)

	// 	for _, file := range dir {
	// 		// We don't use toString() here because we want the underscore
	// 		output := file.Ident.Name + "_" + file.Ident.Version.toString(false)
	// 		fmt.Println(output)
	// 	}
	// }

	// mods := parseCliInput(args, false)
	// for _, mod := range mods {
	// 	fmt.Println(mod.Ident.toString())
	// }
}

func sync(manager *fmm.Manager, args []string) {
	manager.DisableAll()
	fmt.Println("disabled all mods")
	enable(manager, args)
}

func upload(manager *fmm.Manager, files []string) {
	// if apiKey == "" {
	// 	abort("API key not specified.")
	// }
	// if len(files) == 0 {
	// 	abort("no files were provided")
	// }
	// for _, file := range files {
	// 	if err := portalUploadMod(file); err != nil {
	// 		abort("Upload failed:", err)
	// 	}
	// }
}
