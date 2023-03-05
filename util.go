package main

import (
	"fmt"
	"os"

	"github.com/vaughan0/go-ini"
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

func parseConfigFile(path string) error {
	file, err := ini.LoadFile(path)
	if err != nil {
		return err
	}

	if dir, ok := file.Get("", "mods_dir"); ok {
		modsDir = dir
	}

	if username, ok := file.Get("portal", "download_username"); ok {
		downloadUsername = username
	}
	if token, ok := file.Get("portal", "download_token"); ok {
		downloadToken = token
	}

	return nil
}

func usage(usg string, msg ...any) {
	if len(msg) > 0 {
		errorln(msg...)
	}
	errorln("usage:", usg)
	os.Exit(1)
}
