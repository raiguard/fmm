package fmm

import (
	"archive/zip"
	"encoding/json"
	"errors"
	"io"
	"os"
	"path/filepath"
	"strings"
)

type Release struct {
	Name         string
	Dependencies []*Dependency
	Path         string
	Version      Version
}

func releaseFromFile(path string) (*Release, error) {
	info, err := os.Stat(path)
	if err != nil {
		return nil, errors.Join(errors.New("unable to get file info"), err)
	}
	filename := filepath.Base(path)
	var infoJson infoJson
	if info.Mode().IsRegular() {
		infoJson, err = readZipInfoJson(path)
	} else if info.IsDir() || isSymlink(info) {
		file, err := os.Open(filepath.Join(path, "info.json"))
		if err == nil {
			infoJson, err = readInfoJson(file)
		}
	}

	if err != nil {
		return nil, errors.Join(errors.New("error when parsing info.json"), err)
	}

	ident := NewModIdent(filename)
	if ident.Name == infoJson.Name && infoJson.Version.Cmp(ident.Version) != VersionEq {
		return nil, errors.New("invalid release filename")
	}

	return &Release{
		infoJson.Name,
		infoJson.Dependencies,
		filename,
		infoJson.Version,
	}, nil
}

type infoJson struct {
	Dependencies    []*Dependency `json:"dependencies"`
	Name            string        `json:"name"`
	Version         Version       `json:"version"`
	FactorioVersion Version       `json:"factorio_version"`
}

func isSymlink(info os.FileInfo) bool {
	return info.Mode()&os.ModeSymlink > 0
}

func readInfoJson(rc io.ReadCloser) (infoJson, error) {
	var infoJson infoJson

	content, err := io.ReadAll(rc)
	if err != nil {
		return infoJson, err
	}

	err = json.Unmarshal(content, &infoJson)
	return infoJson, err
}

func readZipInfoJson(path string) (infoJson, error) {
	r, err := zip.OpenReader(path)
	if err != nil {
		return infoJson{}, err
	}

	var file *zip.File

	for _, existing := range r.File {
		parts := strings.Split(existing.Name, "/")
		if len(parts) == 2 && parts[1] == "info.json" {
			file = existing
			break
		}
	}

	if file == nil {
		return infoJson{}, errors.New("could not locate info.json file")
	}

	rc, err := file.Open()
	if err != nil {
		return infoJson{}, err
	}
	defer rc.Close()

	return readInfoJson(rc)
}
