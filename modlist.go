package main

import (
	"encoding/json"
	"io/fs"
	"os"
)

type modlist struct {
	Mods []modlistMod `json:"mods"`
	path string
}

type modlistMod struct {
	Name    string  `json:"name"`
	Enabled bool    `json:"enabled"`
	Version *string `json:"version,omitempty"`
}

func newModlist(path string) (*modlist, error) {
	file, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	list := modlist{path: path}
	err = json.Unmarshal(file, &list)
	if err != nil {
		return nil, err
	}
	return &list, nil
}

func (l *modlist) isEnabled(name string) bool {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name == name {
			return mod.Enabled
		}
	}
	return false
}

func (l *modlist) save() error {
	marshaled, err := json.MarshalIndent(l, "", "  ")
	if err != nil {
		return err
	}
	err = os.WriteFile(l.path, marshaled, fs.ModeExclusive)
	return err
}

func (l *modlist) disable(name string) {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name != name {
			continue
		}
		mod.Enabled = false
		break
	}
}

func (l *modlist) enable(name string, version *version) {
	var versionStr *string
	if version != nil {
		output := version.toString(false)
		versionStr = &output
	}
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name != name {
			continue
		}
		mod.Enabled = true
		mod.Version = versionStr
		return
	}
	// Mod was not found, so add it
	mod := modlistMod{Name: name, Enabled: true, Version: versionStr}
	l.Mods = append(l.Mods, mod)
}

func (l *modlist) remove(name string) {
	for i := range l.Mods {
		mod := &l.Mods[i]
		if mod.Name != name {
			continue
		}
		// Why does Go make me do this
		l.Mods = append(l.Mods[:i], l.Mods[i+1:]...)
	}
}
