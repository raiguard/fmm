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
	fmt.Fprintln(os.Stderr, "usage: fmm [-c <file>] [-d <directory>] <clean | disable | enable | install | publish | query | sync> args...")
	os.Exit(1)
}

func main() {
	_, index, err := getopt.Getopts(os.Args, "c:d:")
	if err != nil {
		usage(err)
	}
	args := os.Args[index:]
	if len(args) == 0 {
		usage()
	}

	var task func(args []string)
	op := args[0]
	switch op {
	case "clean", "c":
		panic("clean subcommand is not yet implemented")
	case "disable", "d":
		panic("disable subcommand is not yet implemented")
	case "enable", "e":
		panic("enable subcommand is not yet implemented")
	case "install", "i":
		panic("install subcommand is not yet implemented")
	case "publish", "p":
		panic("publish subcommand is not yet implemented")
	case "query", "q":
		panic("query subcommand is not yet implemented")
	case "sync", "s":
		panic("sync subcommand is not yet implemented")
	default:
		usage("fmm: unknown operation", op)
	}

	task(args)
}
