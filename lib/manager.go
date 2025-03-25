package fmm

import (
	"cmp"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"slices"
)

// Manager manages mdos for a given game directory. A game directory is
// considered valid if it has either a config-path.cfg file or a
// config/config.ini file.
type Manager struct {
	DoSave bool
	Portal ModPortal

	gamePath         string
	internalModsPath string
	modListJsonPath  string
	modSettingsPath  string
	modsPath         string

	mods map[string]*Mod

	modSettings *ModSettings
}

type PlayerData struct {
	Token    string
	Username string
}

// Creates a new Manager for the given game directory. A game directory is
// considered valid if it has either a config-path.cfg file or a
// config/config.ini file. The player's username and token will
// be automatically retrieved from `player-data.json` if it exists.
func NewManager(gamePath string, modsPath string) (*Manager, error) {
	if !entryExists(gamePath, "data", "base", "info.json") || !entryExists(modsPath) {
		return nil, ErrInvalidGameDirectory
	}

	m := Manager{
		DoSave: true,
		Portal: ModPortal{
			downloadPath: modsPath,
			mods:         map[string]*PortalModInfo{},
			server:       "https://mods.factorio.com",
		},

		gamePath:         gamePath,
		internalModsPath: filepath.Join(gamePath, "data"),
		modListJsonPath:  filepath.Join(modsPath, "mod-list.json"),
		modsPath:         modsPath,
		modSettingsPath:  filepath.Join(modsPath, "mod-settings.dat"),
		mods:             map[string]*Mod{},
	}

	if err := m.readPlayerData(); err != nil {
		return nil, errors.Join(errors.New("unable to get player data"), err)
	}

	if !entryExists(m.modsPath) {
		if err := os.Mkdir("mods", 0755); err != nil {
			return nil, errors.Join(errors.New("failed to create mods directory"), err)
		}
	}

	if err := m.parseInternalMods(); err != nil {
		return nil, errors.Join(errors.New("error parsing internal mods"), err)
	}

	if err := m.parseMods(); err != nil {
		return nil, errors.Join(errors.New("error parsing mods"), err)
	}

	for _, mod := range m.mods {
		slices.SortFunc(mod.releases, func(a *Release, b *Release) int {
			switch a.Version.Cmp(&b.Version) {
			case VersionLt:
				return -1
			case VersionGt:
				return 1
			case VersionEq:
				return 0
			// Should be unreachable
			default:
				return 0
			}
		})
	}

	if err := m.parseModList(); err != nil {
		return nil, errors.Join(errors.New("error parsing mod-list.json"), err)
	}

	if base, _ := m.GetMod("base"); base != nil {
		m.Portal.baseVersion = &base.GetLatestRelease().Version
	}

	if entryExists(m.modSettingsPath) {
		file, err := os.Open(m.modSettingsPath)
		if err != nil {
			return nil, errors.Join(errors.New("error parsing mod-settings.dat"), err)
		}
		r := newDatReader(file)
		m.modSettings = ptr(r.ReadModSettings())
	}

	return &m, nil
}

// Add downloads and enables the given mod. Returns the version that was added.
func (m *Manager) Add(mod ModIdent) (*Version, error) {
	ver, err := m.Enable(mod)
	if err == nil {
		return ver, nil
	}

	if !errors.Is(err, ErrModNotFoundLocal) && !errors.Is(err, ErrNoCompatibleRelease) {
		return nil, err
	}

	filepath, err := m.Portal.DownloadRelease(mod.Name, mod.Version)
	if err != nil {
		return nil, err
	}
	release, err := releaseFromFile(filepath)
	if err != nil {
		return nil, err
	}
	m.addRelease(release, false)
	m.Enable(mod)
	return &release.Version, nil
}

// Requests the mod to be disabled.
func (m *Manager) Disable(modName string) error {
	mod, err := m.GetMod(modName)
	if err != nil {
		return err
	}
	if mod.Enabled == nil {
		return ErrModAlreadyDisabled
	}
	mod.Enabled = nil
	return nil
}

// Requests all non-internal mods to be disabled.
func (m *Manager) DisableAll() {
	for _, mod := range m.mods {
		// base is the only mod that is always enabled by default
		if mod.Name != "base" {
			mod.Enabled = nil
		}
	}
}

// Enable the given mod, if it exists. If version is nil, it will default to
// the newest local release. Returns the version that was enabled, if any.
func (m *Manager) Enable(ident ModIdent) (*Version, error) {
	mod, err := m.GetMod(ident.Name)
	if err != nil {
		return nil, err
	}
	release := mod.GetRelease(ident.Version)
	if release == nil {
		return nil, ErrNoCompatibleRelease
	}
	if mod.Enabled != nil && *mod.Enabled == release.Version {
		return nil, nil
	}
	toEnable := &release.Version
	mod.Enabled = toEnable
	return toEnable, nil
}

// Retrieves the corresponding Mod object.
func (m *Manager) GetMod(name string) (*Mod, error) {
	mod := m.mods[name]
	if mod == nil {
		return nil, ErrModNotFoundLocal
	}
	return mod, nil
}

// GetMods returns a list of the mods managed by this Manager.
func (m *Manager) GetMods() []ModIdent {
	mods := []ModIdent{}
	for _, mod := range m.mods {
		for _, release := range mod.releases {
			mods = append(mods, ModIdent{Name: mod.Name, Version: &release.Version})
		}
	}
	return mods
}

// GetLatestMods gets a list of the newest mods managed by this Manager.
func (m *Manager) GetLatestMods() []ModIdent {
	mods := []ModIdent{}
	for _, mod := range m.mods {
		mods = append(mods, ModIdent{mod.Name, &mod.releases[len(mod.releases)-1].Version})
	}
	return mods
}

// Applies the requested modifications and saves to mod-list.json.
func (m *Manager) Save() error {
	if !m.DoSave {
		return nil
	}
	var ModListJson ModListJson
	for name, mod := range m.mods {
		ModListJson.Mods = append(ModListJson.Mods, ModListJsonMod{
			Name:       name,
			Enabled:    mod.Enabled != nil,
			Version:    mod.Enabled,
			isInternal: mod.isInternal,
		})
	}

	slices.SortFunc(ModListJson.Mods, func(a, b ModListJsonMod) int {
		if a.isInternal != b.isInternal {
			if a.isInternal {
				return -1
			} else {
				return 1
			}
		}
		if a.Name != b.Name {
			return cmp.Compare(a.Name, b.Name)
		}
		switch a.Version.Cmp(b.Version) {
		case VersionLt:
			return -1
		case VersionGt:
			return 1
		default:
			return 0
		}
	})

	marshaled, err := json.MarshalIndent(ModListJson, "", "  ")
	if err != nil {
		return err
	}
	err = os.WriteFile(m.modListJsonPath, marshaled, 0666)
	if err != nil {
		return errors.Join(errors.New("failed to write mod-list.json"), err)
	}
	if m.modSettings != nil {
		file, err := os.Create(m.modSettingsPath)
		if err != nil {
			return errors.Join(errors.New("failed to open mod-settings.dat"), err)
		}
		w := newDatWriter(file)
		w.WriteModSettings(m.modSettings)
		w.writer.Flush()
		file.Close()
	}
	return nil
}

// Returns the current upload API key.
func (m *Manager) GetApiKey() string {
	return m.Portal.apiKey
}

// Returns true if the Manager has an upload API key.
func (m *Manager) HasApiKey() bool {
	return m.Portal.apiKey != ""
}

// Sets the API key used for mod uploading.
func (m *Manager) SetApiKey(key string) {
	m.Portal.apiKey = key
}

// Returns the current player data.
func (m *Manager) GetPlayerData() PlayerData {
	return m.Portal.playerData
}

// Returns true if the Manager has valid player data.
func (m *Manager) HasPlayerData() bool {
	return m.Portal.playerData.Token != "" && m.Portal.playerData.Username != ""
}

// Sets the player data used for downloading mods. The player data will be
// automatically retrieved from the game directory if it is available.
func (m *Manager) SetPlayerData(playerData PlayerData) {
	m.Portal.playerData = playerData
}

func (m *Manager) addRelease(release *Release, isInternal bool) {
	mod := m.mods[release.Name]
	if mod == nil {
		mod = &Mod{
			Name:       release.Name,
			releases:   []*Release{},
			isInternal: isInternal,
		}
		m.mods[release.Name] = mod
	}
	mod.releases = append(mod.releases, release)
}

func (m *Manager) parseModList() error {
	m.Enable(ModIdent{Name: "base"})
	mlj, err := ParseModListJson(m.modListJsonPath)
	if err != nil {
		return err
	}
	if mlj == nil {
		m.Enable(ModIdent{Name: "base"})
		return nil
	}

	for _, modEntry := range mlj.Mods {
		if !modEntry.Enabled {
			continue
		}
		mod := m.mods[modEntry.Name]
		if mod == nil {
			continue
		}
		if release := mod.GetRelease(modEntry.Version); release != nil {
			enabled := release.Version
			mod.Enabled = &enabled
		}
	}

	return nil
}

func (m *Manager) parseInternalMods() error {
	entries, err := os.ReadDir(m.internalModsPath)
	if err != nil {
		return errors.Join(errors.New("could not read internal mods directory"), err)
	}

	for _, entry := range entries {
		filename := entry.Name()
		if filename == "core" || !entry.Type().IsDir() {
			continue
		}
		// Not all directories are necessarily mods
		if _, err := os.Stat(filepath.Join(m.internalModsPath, filename, "info.json")); err != nil {
			continue
		}
		release, err := releaseFromFile(filepath.Join(m.internalModsPath, filename))
		if err != nil {
			return errors.Join(errors.New(fmt.Sprint("unable to parse ", filename)), err)
		}
		m.addRelease(release, true)
	}

	return nil
}

func (m *Manager) parseMods() error {
	entries, err := os.ReadDir(m.modsPath)
	if err != nil {
		return errors.Join(errors.New("could not read mods directory"), err)
	}

	for _, entry := range entries {
		filename := entry.Name()
		if filename == "mod-list.json" || filename == "mod-settings.dat" {
			continue
		}
		release, err := releaseFromFile(filepath.Join(m.modsPath, filename))
		if err != nil {
			return errors.Join(errors.New(fmt.Sprint("invalid mod ", filename)), err)
		}
		m.addRelease(release, false)
	}

	return nil
}

func (m *Manager) readPlayerData() error {
	playerDataJsonPath := filepath.Join(m.gamePath, "player-data.json")
	if !entryExists(playerDataJsonPath) {
		return nil
	}

	data, err := os.ReadFile(playerDataJsonPath)
	if err != nil {
		return errors.Join(errors.New("unable to read player-data.json"), err)
	}
	var playerDataJson playerDataJson
	err = json.Unmarshal(data, &playerDataJson)
	if err != nil {
		return errors.Join(errors.New("invalid player-data.json format"), err)
	}
	if playerDataJson.ServiceToken != nil {
		m.Portal.playerData.Token = *playerDataJson.ServiceToken
	}
	if playerDataJson.ServiceUsername != nil {
		m.Portal.playerData.Username = *playerDataJson.ServiceUsername
	}

	return nil
}

func entryExists(pathParts ...string) bool {
	_, err := os.Stat(filepath.Join(pathParts...))
	return err == nil
}

func (m *Manager) ExpandDependencies(mods []ModIdent, fetchFromPortal bool) []ModIdent {
	visited := map[string]bool{}
	toVisit := []Dependency{}
	for _, mod := range mods {
		toVisit = append(toVisit, Dependency{
			Name:    mod.Name,
			Version: mod.Version,
			Kind:    DependencyRequired,
			Req:     VersionEq,
		})
	}
	output := []ModIdent{}

	for i := 0; i < len(toVisit); i += 1 {
		dep := toVisit[i]
		if visited[dep.Name] {
			continue
		}
		visited[dep.Name] = true
		var ident *ModIdent
		var deps []*Dependency
		mod, err := m.GetMod(dep.Name)
		if err != nil && err != ErrModNotFoundLocal {
			fmt.Println(err)
		}
		if mod != nil {
			release := mod.GetMatchingRelease(&dep)
			if release != nil {
				ident = &ModIdent{Name: release.Name, Version: &release.Version}
				deps = release.Dependencies
			}
		}
		if ident == nil && fetchFromPortal {
			var release *PortalModRelease
			release, err = m.Portal.GetMatchingRelease(&dep)
			if err == nil {
				ident = &ModIdent{dep.Name, &release.Version}
				deps = release.InfoJson.Dependencies
			}
		}
		if err != nil {
			fmt.Println(err)
			continue
		}
		if ident == nil {
			panic("Unreachable")
		}
		output = append(output, *ident)
		for _, dep := range deps {
			if dep.Kind == DependencyRequired || dep.Kind == DependencyNoLoadOrder {
				toVisit = append(toVisit, *dep)
			}
		}
	}

	return output
}

func (m *Manager) MergeStartupModSettings(input PropertyTree) error {
	if input == nil {
		return nil
	}
	if _, ok := input.(*PropertyTreeNone); ok {
		return nil
	}
	inputSettings, ok := input.(*PropertyTreeDict)
	if !ok {
		panic("input mod settings have invalid structure")
	}
	if m.modSettings == nil {
		base, err := m.GetMod("base")
		if err != nil {
			return err
		}
		m.modSettings = &ModSettings{
			MapVersion: base.GetLatestRelease().Version,
			Settings: &PropertyTreeDict{
				"startup":          &PropertyTreeDict{},
				"runtime-global":   &PropertyTreeDict{},
				"runtime-per-user": &PropertyTreeDict{},
			},
		}
	}

	modSettings, ok := m.modSettings.Settings.(*PropertyTreeDict)
	if !ok {
		panic("mod settings have invalid structure")
	}
	startupSettings, ok := (*modSettings)["startup"].(*PropertyTreeDict)
	if !ok {
		panic("mod startup settings have invalid structure")
	}
	for key, value := range *inputSettings {
		(*startupSettings)[key] = value
	}

	m.DoSave = true

	return nil
}

func (m *Manager) CheckDownloadUpdates(mods []ModIdent) {
	if len(mods) == 0 {
		mods = m.GetLatestMods()
	}
	for _, mod := range mods {
		info, err := m.Portal.GetModInfo(mod.Name)
		if err != nil {
			fmt.Println(err)
			continue
		}
		if len(info.Releases) == 0 {
			continue // WTF?
		}
		if info.Releases[len(info.Releases)-1].Version.Cmp(mod.Version) == VersionGt {
			m.Portal.DownloadLatestRelease(mod.Name)
		}
	}
}
