package fmm

import (
	"strings"
)

// A small representation of a mod, with an optional version.
type ModIdent struct {
	Name    string
	Version *Version
}

// Returns a ModIdent parsed from an input string with the format of 'name',
// 'name_version', or 'name_version.zip'.
func NewModIdent(input string) ModIdent {
	input = strings.TrimSuffix(input, ".zip")
	parts := strings.Split(input, "_")
	if len(parts) == 1 {
		return ModIdent{input, nil}
	}

	name := strings.Join(parts[:len(parts)-1], "_")
	version, err := NewVersion(parts[len(parts)-1])
	if err != nil {
		return ModIdent{input, nil}
	}
	return ModIdent{name, version}
}

// Returns a string in the format of 'name' or 'name_version'.
func (i *ModIdent) ToString() string {
	if i.Version != nil {
		return i.Name + " " + i.Version.ToString(false)
	}
	return i.Name
}
