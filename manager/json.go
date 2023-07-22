package manager

import "strings"

type modListJson struct {
	Mods modListJsonMods `json:"mods"`
}

type modListJsonMods []modListJsonMod

// Implementations for sorting interface
// TODO: Use Go 1.21 `slices` module once it is released
func (m modListJsonMods) Len() int {
	return len(m)
}
func (m modListJsonMods) Swap(i, j int) {
	m[i], m[j] = m[j], m[i]
}
func (m modListJsonMods) Less(i, j int) bool {
	modi, modj := &m[i], &m[j]
	if internalMods[modi.Name] != internalMods[modj.Name] {
		return internalMods[modi.Name]
	}
	if modi.Name != modj.Name {
		return strings.ToLower(modi.Name) < strings.ToLower(modj.Name)
	}
	return modi.Version.Cmp(modj.Version) == VersionLt
}

type modListJsonMod struct {
	Name    string   `json:"name"`
	Enabled bool     `json:"enabled"`
	Version *Version `json:"version,omitempty"`
}

type playerDataJson struct {
	ServiceToken    *string `json:"service-token"`
	ServiceUsername *string `json:"service-username"`
}
