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

func getFromEnvOrConfig(env string, file ini.File, section string, key string) string {
	if value := os.Getenv(env); value != "" {
		return value
	} else if value, ok := file.Get(section, key); ok {
		return value
	}
	return ""
}

func parseConfig(path string) error {
	file, err := ini.LoadFile(path)
	if err != nil {
		return err
	}

	if dir, ok := file.Get("", "mods_dir"); ok {
		modsDir = dir
	}

	apiKey = getFromEnvOrConfig("FACTORIO_API_KEY", file, "portal", "api_key")
	downloadToken = getFromEnvOrConfig("FACTORIO_TOKEN", file, "portal", "token")
	downloadUsername = getFromEnvOrConfig("FACTORIO_USERNAME", file, "portal", "username")

	return nil
}
