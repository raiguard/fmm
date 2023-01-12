package main

import (
	"encoding/json"
	"io/fs"
	"os"
)

type ModList struct {
	Mods []ModListMod `json:"mods"`
	path string
}

type ModListMod struct {
	Name    string  `json:"name"`
	Enabled bool    `json:"enabled"`
	Version *string `json:"version,omitempty"`
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

func (l *ModList) disable(name string) {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name == name {
			mod.Enabled = false
			break
		}
	}
}

func (l *ModList) enable(name string, version *Version) {
	var versionStr *string
	if version != nil {
		output := version.toString(false)
		versionStr = &output
	}
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name == name {
			mod.Enabled = true
			mod.Version = versionStr
			return
		}
	}
	// Mod was not found, so add it
	mod := ModListMod{Name: name, Enabled: true, Version: versionStr}
	l.Mods = append(l.Mods, mod)
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
