package cli

import (
	"fmt"
	"os"
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
