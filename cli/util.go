package cli

import (
	"fmt"
	"os"
	"strings"

	fmm "github.com/raiguard/fmm/lib"
)

func abort(msg ...any) {
	errorln(msg...)
	os.Exit(1)
}

func errorln(msg ...any) {
	fmt.Fprintln(os.Stderr, msg...)
}

func errorf(format string, msg ...any) {
	fmt.Fprintf(os.Stderr, format, msg...)
}

func printUsage(msg ...any) {
	if len(msg) > 0 {
		errorln(msg...)
	}
	errorln(usageStr)
	os.Exit(1)
}

func getMods(args []string) ([]fmm.ModIdent, fmm.PropertyTree) {
	var mods []fmm.ModIdent
	var settings fmm.PropertyTree

	for _, input := range args {
		var thisMods []fmm.ModIdent
		var err error
		if strings.HasSuffix(input, ".zip") {
			var fileInfo fmm.SaveFileInfo
			fileInfo, err = fmm.ParseSaveFile(input)
			if settings == nil {
				settings = fileInfo.ModSettings
			}
			thisMods = fileInfo.Mods
		} else if strings.HasSuffix(input, ".log") {
			thisMods = fmm.ParseLogFile(input)
		} else if strings.HasSuffix(input, ".json") {
			var mlj *fmm.ModListJson
			mlj, err = fmm.ParseModListJson(input)
			if mlj != nil {
				for _, mod := range mlj.Mods {
					if mod.Enabled {
						thisMods = append(thisMods, fmm.ModIdent{Name: mod.Name, Version: mod.Version})
					}
				}
			}
		} else if strings.HasPrefix(input, "!") {
			// TODO: Mod set
		} else {
			thisMods = append(thisMods, fmm.NewModIdent(input))
		}
		if err != nil {
			fmt.Println(err)
			continue
		}
		mods = append(mods, thisMods...)
	}

	return mods, settings
}
