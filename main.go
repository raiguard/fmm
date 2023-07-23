package main

import (
	"os"

	"github.com/raiguard/fmm/cli"
)

func main() {
	cli.Run(os.Args[1:])
}
