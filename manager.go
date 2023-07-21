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

var internalMods = map[string]bool{
	"base": true,
	"core": true,
}

type Manager struct {
	DoSave bool

	apiKey           string
	downloadToken    string
	downloadUsername string
	gamePath         string
	mods             mods
	modsPath         string
	modListJsonPath  string
	states           map[string]*StateData
}

type StateData struct {
	Enabled bool
	Version *Version
}

func NewManager(gamePath string) (*Manager, error) {
	if !isFactorioDir(gamePath) {
		return nil, errors.New("Invalid Factorio data directory")
	}

	m := Manager{
		DoSave:          true,
		gamePath:        gamePath,
		modListJsonPath: path.Join(gamePath, "mods", "mod-list.json"),
		mods:            []Mod{},
		modsPath:        path.Join(gamePath, "mods"),
		states:          map[string]*StateData{},
	}

	if err := m.getPlayerData(); err != nil {
		return nil, errors.Join(errors.New("Unable to get player data"), err)
	}

	modListJsonPath := path.Join(m.modsPath, "mod-list.json")
	if !entryExists(m.modsPath) {
		if err := os.Mkdir("mods", 0755); err != nil {
			return nil, errors.Join(errors.New("Failed to create mods directory"), err)
		}
		// TODO: Auto-create mod-list.json
		// if err := os.WriteFile(modListJsonPath, ; err != nil {
		// 	return m, errors.Join(errors.New("Failed to create mod-list.json"), err)
		// }
	}

	modListJsonData, err := os.ReadFile(modListJsonPath)
	if err != nil {
		return nil, errors.Join(errors.New("Error reading mod-list.json"), err)
	}
	var modListJson modListJson
	if err = json.Unmarshal(modListJsonData, &modListJson); err != nil {
		return nil, errors.Join(errors.New("Error parsing mod-list.json"), err)
	}
	for _, modEntry := range modListJson.Mods {
		stateData := StateData{Enabled: modEntry.Enabled}
		if modEntry.Version != nil {
			*stateData.Version = *modEntry.Version
		}
		m.states[modEntry.Name] = &stateData
	}

	if err := m.parseMods(); err != nil {
		return nil, errors.Join(errors.New("Error parsing mods"), err)
	}

	return &m, nil
}

func (m *Manager) Disable(modName string) {
	if stateData, ok := m.states[modName]; ok {
		stateData.Enabled = false
	}
}

func (m *Manager) DisableAll() {
	for name, stateData := range m.states {
		if !internalMods[name] {
			stateData.Enabled = false
		}
	}
}

func (m *Manager) Enable(mod ModIdent) error {
	stateData, ok := m.states[mod.Name]
	if !ok {
		return errors.New(fmt.Sprintf("Unable to enable %s: does not exist in the mods directory", mod.toString()))
	}

	stateData.Enabled = true
	if mod.Version != nil {
		*stateData.Version = *mod.Version
	}

	return nil
}

func (m *Manager) Save() error {
	if !m.DoSave {
		return nil
	}
	var ModListJson modListJson
	for name, stateData := range m.states {
		ModListJson.Mods = append(ModListJson.Mods, modListJsonMod{
			Name:    name,
			Enabled: stateData.Enabled,
			Version: stateData.Version,
		})
	}
	sort.Sort(ModListJson.Mods)
	marshaled, err := json.MarshalIndent(ModListJson, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(m.modListJsonPath, marshaled, fs.ModeExclusive)
}

func (m *Manager) getPlayerData() error {
	playerDataJsonPath := path.Join(m.gamePath, "player-data.json")
	if !entryExists(playerDataJsonPath) {
		return nil
	}

	data, err := os.ReadFile(playerDataJsonPath)
	if err != nil {
		return errors.Join(errors.New("Unable to read player-data.json"), err)
	}
	var playerDataJson playerDataJson
	err = json.Unmarshal(data, &playerDataJson)
	if err != nil {
		return errors.Join(errors.New("Invalid player-data.json format"), err)
	}
	if playerDataJson.ServiceToken != nil {
		m.downloadToken = *playerDataJson.ServiceToken
	}
	if playerDataJson.ServiceUsername != nil {
		m.downloadUsername = *playerDataJson.ServiceUsername
	}

	return nil
}

func (m *Manager) parseMods() error {
	files, err := os.ReadDir(m.modsPath)
	if err != nil {
		return errors.Join(errors.New("Could not read mods directory"), err)
	}

	for _, file := range files {
		name := file.Name()
		if name == "mod-list.json" || name == "mod-settings.dat" {
			continue
		}
		var ident ModIdent
		fileType := file.Type()
		var deps *[]Dependency
		if fileType.IsDir() || fileType&fs.ModeSymlink > 0 {
			infoJson, err := parseInfoJson(path.Join(m.modsPath, name, "info.json"))
			if err != nil {
				// TODO: Multi-error handling
				errorln(err)
				continue
			}
			ident.Name = infoJson.Name
			ident.Version = &infoJson.Version
			deps = &infoJson.Dependencies
		} else {
			ident = newModIdent(name)
		}
		// TODO: Make optional?
		if _, ok := m.states[ident.Name]; !ok {
			m.states[ident.Name] = &StateData{Enabled: true}
		}
		m.mods = append(m.mods, Mod{
			dependencies: deps,
			Ident:        ident,
			Path:         path.Join(m.modsPath, name),
			Type:         fileType,
		})
	}

	// Sort files so we can reliably get the newest version
	sort.Sort(m.mods)

	return nil
}

func entryExists(pathParts ...string) bool {
	_, err := os.Stat(path.Join(pathParts...))
	return err == nil
}

func isFactorioDir(dir string) bool {
	if !entryExists(dir, "data", "changelog.txt") {
		return false
	}
	if !entryExists(dir, "data", "base", "info.json") {
		return false
	}
	return entryExists(dir, "config-path.ini") || entryExists(dir, "config", "config.ini")
}
