package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path"
	"sort"
)

type Dir struct {
	Files ModFiles
	List  ModList
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
		if fileType.IsDir() || fileType&fs.ModeSymlink > 0 {
			infoJson, err := parseInfoJson(path.Join(dirPath, name, "info.json"))
			if err != nil {
				errorln(err)
				continue
			}
			ident.Name = infoJson.Name
			ident.Version = &infoJson.Version // TODO: Will this preserve InfoJson forever?
		} else {
			ident = newModIdent(name)
		}
		files = append(files, ModFile{
			Ident: ident,
			Path:  name,
			Type:  fileType,
		})
	}

	// Sort files so we can reliably get the newest version
	sort.Sort(files)

	return &Dir{
		Files: files,
		List:  *list,
	}, nil
}

func (d *Dir) find(mod ModIdent) (file *ModFile, entry *ModListMod, err error) {
	// Iterate in reverse to get the newest version first
	for i := len(d.Files) - 1; i >= 0; i-- {
		thisfile := &d.Files[i]
		if thisfile.Ident.Name != mod.Name {
			continue
		}
		if mod.Version == nil || thisfile.Ident.Version.cmp(*mod.Version) == VersionEq {
			file = thisfile
			break
		}
	}
	if file == nil {
		return nil, nil, errors.New(fmt.Sprintf("%s was not found in the mods directory", mod.toString()))
	}

	for i := range d.List.Mods {
		thisentry := &d.List.Mods[i]
		if thisentry.Name == mod.Name {
			entry = thisentry
			break
		}
	}

	if entry == nil {
		entry = d.List.add(mod.Name)
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
	Dependencies []string
	Ident        ModIdent
	Path         string
	Type         fs.FileMode
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
