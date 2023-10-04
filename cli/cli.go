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
commands:
	add     [args...]   Download and enable the given mods and their dependencies
	disable [args...]   Disable the given mods, or all mods if none are given
	enable  [args...]   Enable the given mods and their dependencies
	help                Show usage information
	list    [files...]  List all mods in the mods directory, or in the given save files
	sync    [args...]   Disable all mods, then download and enable the given mods and their dependencies
	upload  [files...]  Upload the given mod zip files to the mod portal`

func Run(args []string) {
	if len(args) == 0 {
		printUsage("no operation was specified")
	}

	var task func(*fmm.Manager, []string)
	switch args[0] {
	case "add", "a":
		task = add
	case "disable", "d":
		task = disable
	case "enable", "e":
		task = enable
	case "help", "h", "-h", "--help", "-help":
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
	args = args[1:]

	manager, err := fmm.NewManager(".")
	if err != nil {
		if !errors.Is(err, fmm.ErrInvalidGameDirectory) {
			abort(err)
		}
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

	stdinStat, _ := os.Stdin.Stat()
	if stdinStat.Mode()&os.ModeNamedPipe > 0 {
		bytes, err := io.ReadAll(os.Stdin)
		if err == nil {
			args = append(args, strings.Split(strings.TrimSpace(string(bytes)), "\n")...)
		}
	}

	task(manager, args)

	if err := manager.Save(); err != nil {
		errorln(errors.Join(errors.New("unable to save modifications"), err))
	}
}

func add(manager *fmm.Manager, args []string) {
	for _, mod := range manager.ExpandDependencies(getMods(args), true) {
		ver, err := manager.Add(mod)
		if err != nil {
			errorf("failed to add %s\n", mod.ToString())
			errorln(err)
		} else if ver != nil {
			fmt.Println("enabled", mod.Name, ver.ToString(false))
		}
	}
}

func disable(manager *fmm.Manager, args []string) {
	if len(args) == 0 {
		manager.DisableAll()
		fmt.Println("disabled all mods")
		return
	}

	for _, mod := range getMods(args) {
		if err := manager.Disable(mod.Name); err != nil {
			errorf("failed to disable %s\n", mod.ToString())
			errorln(err)
		} else {
			fmt.Println("disabled", mod.Name)
		}
	}
}

func enable(manager *fmm.Manager, args []string) {
	for _, mod := range manager.ExpandDependencies(getMods(args), false) {
		ver, err := manager.Enable(mod)
		if err != nil {
			errorf("failed to enable %s\n", mod.ToString())
			errorln(err)
		} else if ver != nil {
			fmt.Println("enabled", mod.Name, ver.ToString(false))
		}
	}
}

func list(manager *fmm.Manager, args []string) {}

func sync(manager *fmm.Manager, args []string) {
	manager.DisableAll()
	fmt.Println("disabled all mods")
	add(manager, args)
}

func upload(manager *fmm.Manager, files []string) {
	for _, file := range files {
		if err := manager.Portal.UploadMod(file); err != nil {
			fmt.Println(err)
		}
	}
}
