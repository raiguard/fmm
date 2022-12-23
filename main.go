package main

import (
	"fmt"
	"os"

	"git.sr.ht/~sircmpwn/getopt"
)

func usage(msg ...any) {
	if len(msg) > 0 {
		fmt.Fprintln(os.Stderr, msg...)
	}
	fmt.Fprintln(os.Stderr, "usage: fmm [-l <file>] <disable | enable> mods...")
	os.Exit(1)
}

func main() {
	opts, index, err := getopt.Getopts(os.Args, "l:")
	if err != nil {
		usage(err)
	}
	args := os.Args[index:]
	if len(args) == 0 {
		usage()
	}

	modlistPath := "mod-list.json"
	for _, opt := range opts {
		switch opt.Option {
		case 'l':
			modlistPath = opt.Value
		}
	}

	list, err := newModlist(modlistPath)
	if err != nil {
		usage(err)
	}

	op := args[0]
	for _, input := range args[1:] {
		mod := newModident(input)
		switch op {
		case "enable", "e":
			list.enable(mod.Name, mod.Version)
		case "disable", "d":
			list.disable(mod.Name)
		default:
			usage("Unrecognized operation: ", op)
		}
	}

	list.save()
}
