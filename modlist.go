package main

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
)

type ModList struct {
	Mods []ModListMod `json:"mods"`
	path string
}

type ModListMod struct {
	Name    string   `json:"name"`
	Enabled bool     `json:"enabled"`
	Version *Version `json:"version,omitempty"`
}

func newModList(path string) (*ModList, error) {
	file, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	list := ModList{path: path}
	err = json.Unmarshal(file, &list)
	if err != nil {
		return nil, err
	}
	return &list, nil
}

func (l *ModList) isEnabled(name string) bool {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name == name {
			return mod.Enabled
		}
	}
	return false
}

func (l *ModList) save() error {
	marshaled, err := json.MarshalIndent(l, "", "  ")
	if err != nil {
		return err
	}
	err = os.WriteFile(l.path, marshaled, fs.ModeExclusive)
	return err
}

func (l *ModList) add(name string) *ModListMod {
	l.Mods = append(l.Mods, ModListMod{name, true, nil})
	return &l.Mods[len(l.Mods)-1]
}

func (l *ModList) disable(name string) {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name == name {
			mod.Enabled = false
			fmt.Println("Disabled", name)
			break
		}
	}
}

func (l *ModList) enable(mod ModIdent) {
	for i := range l.Mods {
		entry := &l.Mods[i]
		if entry.Name == mod.Name {
			entry.Enabled = true
			entry.Version = mod.Version
			fmt.Println("Enabled", mod.toString())
			return
		}
	}
	// Mod was not found, so add it
	entry := ModListMod{Name: mod.Name, Enabled: true, Version: mod.Version}
	l.Mods = append(l.Mods, entry)
}

func (l *ModList) remove(name string) {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name != name {
			continue
		}
		// Why does Go make me do this
		l.Mods = append(l.Mods[:i], l.Mods[i+1:]...)
	}
}
