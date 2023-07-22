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

func getMods(args []string) []fmm.ModIdent {
	var mods []fmm.ModIdent

	for _, input := range args {
		var thisMods []fmm.ModIdent
		var err error
		if strings.HasSuffix(input, ".zip") {
			thisMods, err = fmm.ParseSaveFile(input)
		} else if strings.HasSuffix(input, ".log") {
			thisMods = fmm.ParseLogFile(input)
		} else if strings.HasSuffix(input, ".json") {
			// TODO: mod-list.json
		} else if strings.HasPrefix(input, "!") {
			// TODO: Mod set
		} else {
			thisMods = append(thisMods, fmm.NewModIdent(input))
		}
		if err != nil {
			errorln(err)
			continue
		}
		mods = append(mods, thisMods...)
	}

	return mods
}
