package main

import (
	"archive/zip"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"io/ioutil"
	"os"
	"path"
	"sort"
	"strings"
)

type Dir struct {
	Files ModFiles
	List  ModList
	Path  string
}

func newDir(dirPath string) (*Dir, error) {
	file, err := os.ReadDir(dirPath)
	if err != nil {
		return nil, err
	}

	list, err := newModList(path.Join(dirPath, "mod-list.json"))
	if err != nil {
		// TODO: Create a json?
		return nil, err
	}

	var files ModFiles
	for _, file := range file {
		name := file.Name()
		if name == "mod-list.json" || name == "mod-settings.dat" {
			continue
		}
		var ident ModIdent
		fileType := file.Type()
		var deps *[]Dependency
		if fileType.IsDir() || fileType&fs.ModeSymlink > 0 {
			infoJson, err := parseInfoJson(path.Join(dirPath, name, "info.json"))
			if err != nil {
				errorln(err)
				continue
			}
			ident.Name = infoJson.Name
			ident.Version = &infoJson.Version // TODO: Will this preserve InfoJson forever?
			deps = &infoJson.Dependencies
		} else {
			ident = newModIdent(name)
		}
		files = append(files, ModFile{
			dependencies: deps,
			Ident:        ident,
			Path:         path.Join(dirPath, name),
			Type:         fileType,
		})
	}

	// Sort files so we can reliably get the newest version
	sort.Sort(files)

	return &Dir{
		Files: files,
		List:  *list,
		Path:  dirPath,
	}, nil
}

func (d *Dir) Find(mod Dependency) (file *ModFile, entry *ModListMod, err error) {
	// Iterate in reverse to get the newest version first
	for i := len(d.Files) - 1; i >= 0; i-- {
		thisfile := &d.Files[i]
		if thisfile.Ident.Name != mod.Ident.Name {
			continue
		}
		if mod.Test(&thisfile.Ident) {
			file = thisfile
			break
		}
	}
	if file == nil {
		return nil, nil, errors.New(fmt.Sprintf("%s was not found in the mods directory", mod.Ident.toString()))
	}

	for i := range d.List.Mods {
		thisentry := &d.List.Mods[i]
		if thisentry.Name == mod.Ident.Name {
			entry = thisentry
			break
		}
	}

	if entry == nil {
		entry = d.List.add(mod.Ident.Name)
	}

	return file, entry, nil
}

func (d *Dir) save() {
	err := d.List.save()
	if err != nil {
		abort(err)
	}
}

// Wrapper type with implementations for sorting
type ModFiles []ModFile

func (f ModFiles) Len() int {
	return len(f)
}
func (f ModFiles) Swap(i, j int) {
	f[i], f[j] = f[j], f[i]
}
func (f ModFiles) Less(i, j int) bool {
	modi, modj := f[i].Ident, &f[j].Ident
	if modi.Name != modj.Name {
		return modi.Name < modj.Name
	}
	return modi.Version.cmp(*modj.Version) == VersionLt
}

type ModFile struct {
	dependencies *[]Dependency

	Ident ModIdent
	Path  string
	Type  fs.FileMode
}

func (f *ModFile) Dependencies() (*[]Dependency, error) {
	if f.dependencies != nil {
		return f.dependencies, nil
	}

	if !f.Type.IsRegular() {
		return nil, errors.New("Failed to get dependencies for unzipped mod")
	}

	r, err := zip.OpenReader(f.Path)
	if err != nil {
		return nil, err
	}

	for _, file := range r.File {
		// TODO: Use a regex to get the right one
		if !strings.Contains(file.Name, "info.json") {
			continue
		}
		rc, err := file.Open()
		if err != nil {
			return nil, err
		}
		defer rc.Close()
		content, err := ioutil.ReadAll(rc)
		if err != nil {
			return nil, err
		}

		var unmarshaled InfoJson
		err = json.Unmarshal(content, &unmarshaled)
		if err != nil {
			return nil, err
		}
		f.dependencies = &unmarshaled.Dependencies
	}

	return f.dependencies, nil
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
