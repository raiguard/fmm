package main

import (
	"archive/zip"
	"encoding/json"
	"errors"
	"io"
	"io/fs"
	"os"
	"strings"
)

type Mod struct {
	dependencies *[]Dependency

	Ident ModIdent
	Path  string
	Type  fs.FileMode
}

func (f *Mod) Dependencies() ([]Dependency, error) {
	if f.dependencies != nil {
		return *f.dependencies, nil
	}

	if !f.Type.IsRegular() {
		return nil, errors.New("Failed to get dependencies for unzipped mod")
	}

	r, err := zip.OpenReader(f.Path)
	if err != nil {
		return nil, err
	}

	var file *zip.File
	for _, existing := range r.File {
		if !strings.Contains(existing.Name, "info.json") {
			continue
		}
		parts := strings.Split(existing.Name, "/")
		if len(parts) != 2 || parts[1] != "info.json" {
			continue
		}
		file = existing
	}

	if file == nil {
		return nil, errors.New("Mod does not contain an info.json file")
	}

	rc, err := file.Open()
	if err != nil {
		return nil, err
	}
	defer rc.Close()
	content, err := io.ReadAll(rc)
	if err != nil {
		return nil, err
	}

	var unmarshaled InfoJson
	err = json.Unmarshal(content, &unmarshaled)
	if err != nil {
		return nil, err
	}
	f.dependencies = &unmarshaled.Dependencies

	return *f.dependencies, nil
}

type InfoJson struct {
	Dependencies []Dependency `json:"dependencies"`
	Name         string       `json:"name"`
	Version      Version      `json:"version"`
}

func parseInfoJson(path string) (*InfoJson, error) {
	bytes, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	var unmarshaled InfoJson
	err = json.Unmarshal(bytes, &unmarshaled)
	if err != nil {
		return nil, err
	}

	return &unmarshaled, nil
}
