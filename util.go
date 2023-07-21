package main

import (
	"fmt"
	"os"
	"path"
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

func entryExists(pathParts ...string) bool {
	_, err := os.Stat(path.Join(pathParts...))
	return err == nil
}

func isFactorioDir(dir string) bool {
	if !entryExists(dir, "data", "changelog.txt") {
		return false
	}
	if !entryExists(dir, "data", "base", "info.json") {
		return false
	}
	return entryExists(dir, "config-path.ini") || entryExists(dir, "config", "config.ini")
}
