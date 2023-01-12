package main

import "github.com/vaughan0/go-ini"

type Config struct {
	ModsDir string
}

func newConfig(path string) error {
	file, err := ini.LoadFile(path)
	if err != nil {
		return err
	}

	config = Config{
		ModsDir: "mod-list.json",
	}

	if dir, ok := file.Get("", "mods_dir"); ok {
		config.ModsDir = dir
	}

	return nil
}
