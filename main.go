package main

import (
	"fmt"
	"os"

	"git.sr.ht/~sircmpwn/getopt"
	"github.com/adrg/xdg"
)

var (
	apiKey           string = ""
	configPath       string = "./fmm.ini"
	downloadToken    string = ""
	downloadUsername string = ""
	modsDir          string = "."
)

const (
	disableUsage  string = "fmm disable [mods...]"
	downloadUsage string = "fmm download [mods...]"
	enableUsage   string = "fmm enable [mods...]"
	mainUsage     string = "fmm [-c <file>] <disable | download | enable | upload> [args...]"
	uploadUsage   string = "fmm upload [files...]"
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
	if err := parseConfig(configPath); err != nil {
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
	case "download", "dl":
		task = download
	case "help", "h":
		usage(mainUsage)
	case "sync", "s":
		task = sync
	case "upload", "ul":
		task = upload
	default:
		usage(mainUsage, fmt.Sprintf("%s: unknown operation %s", os.Args[0], args[0]))
	}
	task(args[1:])
}
