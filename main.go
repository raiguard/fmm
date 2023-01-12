package main

import (
	"fmt"
	"os"

	"git.sr.ht/~sircmpwn/getopt"
	"github.com/adrg/xdg"
)

var (
	configPath string = "./fmm.ini"
	modsDir    string = "."
)

const (
	disableUsage string = "fmm disable [mods...]"
	enableUsage  string = "fmm enable <mods...>"
	mainUsage    string = "fmm [-c <file>] <disable | enable> args..."
)

func main() {
	opts, index, err := getopt.Getopts(os.Args, "c:h")
	if err != nil {
		usage(mainUsage, err)
	}

	xdgConfigPath, err := xdg.ConfigFile("fmm/fmm.ini")
	if err == nil {
		configPath = xdgConfigPath
	}

	for _, opt := range opts {
		switch opt.Option {
		case 'c':
			configPath = opt.Value
		case 'h':
			usage(mainUsage)
		}
	}
	if err := parseConfigFile(configPath); err != nil {
		abort("could not parse config file:", err)
	}

	args := os.Args[index:]
	if len(args) == 0 {
		usage(mainUsage, "no operation was specified")
	}

	var task func([]string)
	switch args[0] {
	case "disable", "d":
		task = disable
	case "enable", "e":
		task = enable
	default:
		usage(mainUsage, fmt.Sprintf("%s: unknown operation %s", os.Args[0], args[0]))
	}
	task(args[1:])
}
