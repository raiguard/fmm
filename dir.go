package main

import (
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

func newDir(name string) (*Dir, error) {
	file, err := os.ReadDir(name)
	if err != nil {
		return nil, err
	}

	list, err := newModList(path.Join(name, "mod-list.json"))
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
		fileType := file.Type()
		// TODO: Extract info.json
		if !fileType.IsRegular() {
			continue
		}
		files = append(files, ModFile{
			// TODO: Guarantee version
			Ident: newModIdent(name),
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
	// Dependencies []Dependency
	Ident ModIdent
	Path  string
	Type  fs.FileMode
}
