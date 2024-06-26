package cli

import (
	"cmp"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"slices"
	"strings"

	fmm "github.com/raiguard/fmm/lib"
)

const usageStr string = `usage: fmm <command> [args...]
commands:
  add     [args...]   Download and enable the given mods and their dependencies.
  disable [args...]   Disable the given mods, or all mods if none are given.
  enable  [args...]   Enable the given mods and their dependencies.
  help                Show usage information.
  list    [files...]  List all mods in the mods directory, or in the given save files.
  sync    [args...]   Disable all mods, then download and enable the given mods and their dependencies.
                      If a save file is provided, merge startup mod settings with the settings contained in that save.
  update  [args...]   Update the given mods, or all mods if none are given.
  upload  [files...]  Upload the given mod zip files to the mod portal.`

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
	case "update", "u":
		task = update
	case "upload", "ul":
		task = upload
	default:
		printUsage("unrecognized operation", args[0])
	}
	args = args[1:]

	manager, err := fmm.NewManager(".", filepath.Join(".", "mods"))
	if err != nil {
		if !errors.Is(err, fmm.ErrInvalidGameDirectory) {
			abort(err)
		}
		gamePath := os.Getenv("FACTORIO_PATH")
		modsPath := os.Getenv("FACTORIO_MODS_PATH")
		if modsPath == "" {
			modsPath = filepath.Join(gamePath, "mods")
		}
		manager, err = fmm.NewManager(gamePath, modsPath)
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
	mods, _ := getMods(args)
	for _, mod := range manager.ExpandDependencies(mods, true) {
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

	mods, _ := getMods(args)
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
	mods, _ := getMods(args)
	for _, mod := range manager.ExpandDependencies(mods, false) {
		ver, err := manager.Enable(mod)
		if err != nil {
			errorf("failed to enable %s\n", mod.ToString())
			errorln(err)
		} else if ver != nil {
			fmt.Println("enabled", mod.Name, ver.ToString(false))
		}
	}
}

func list(manager *fmm.Manager, args []string) {
	mods := []fmm.ModIdent{}
	if len(args) == 0 {
		mods = manager.GetMods()
	} else {
		for _, filepath := range args {
			fileInfo, err := fmm.ParseSaveFile(filepath)
			if err != nil {
				fmt.Println(err)
			}
			mods = append(mods, fileInfo.Mods...)
		}
	}
	slices.SortFunc(mods, func(a fmm.ModIdent, b fmm.ModIdent) int {
		if a.Name != b.Name {
			return cmp.Compare(a.Name, b.Name)
		}
		switch a.Version.Cmp(b.Version) {
		case fmm.VersionLt:
			return -1
		case fmm.VersionGt:
			return 1
		case fmm.VersionEq:
			return 0
		// Should be unreachable
		default:
			return 0
		}
	})
	for _, mod := range mods {
		fmt.Println(mod.ToString())
	}
}

func sync(manager *fmm.Manager, args []string) {
	manager.DisableAll()
	fmt.Println("disabled all mods")
	mods, settings := getMods(args)
	for _, mod := range manager.ExpandDependencies(mods, true) {
		ver, err := manager.Add(mod)
		if err != nil {
			errorf("failed to add %s\n", mod.ToString())
			errorln(err)
		} else if ver != nil {
			fmt.Println("enabled", mod.Name, ver.ToString(false))
		}
	}
	if settings != nil {
		manager.MergeStartupModSettings(settings)
		fmt.Println("synced startup mod settings")
	}
}

func update(manager *fmm.Manager, args []string) {
	mods, _ := getMods(args)
	manager.CheckDownloadUpdates(mods)
}

func upload(manager *fmm.Manager, files []string) {
	for _, file := range files {
		if err := manager.Portal.UploadMod(file); err != nil {
			fmt.Println(err)
		}
	}
}
