package main

import (
	"fmt"
	"os"
	"path"

	"git.sr.ht/~sircmpwn/getopt"
	"github.com/adrg/xdg"
)

var config Config

func usage(msg ...any) {
	if len(msg) > 0 {
		fmt.Fprintln(os.Stderr, msg...)
	}
	fmt.Fprintln(os.Stderr, "usage: fmm [-c <file>] <disable | enable> mods...")
	os.Exit(1)
}

func main() {
	opts, index, err := getopt.Getopts(os.Args, "c:")
	if err != nil {
		usage(err)
	}
	args := os.Args[index:]
	if len(args) == 0 {
		usage()
	}

	configPath, err := xdg.ConfigFile("fmm/fmm.ini")
	if err != nil {
		usage(err)
	}

	for _, opt := range opts {
		switch opt.Option {
		case 'c':
			configPath = opt.Value
		}
	}

	parseConfig(configPath)

	list, err := newModlist(path.Join(config.ModsDir, "mod-list.json"))
	if err != nil {
		usage(err)
	}

	op := args[0]
	for _, input := range args[1:] {
		mod := newModident(input)
		switch op {
		case "disable", "d":
			list.disable(mod.Name)
			fmt.Println("Disabled", mod.toString())
		case "enable", "e":
			list.enable(mod.Name, mod.Version)
			fmt.Println("Enabled", mod.toString())
		default:
			usage("Unrecognized operation: ", op)
		}
	}

	list.save()
}
