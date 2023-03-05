package main

import (
	"strings"
)

type ModIdent struct {
	Name    string
	Version *Version
}

func newModIdent(input string) ModIdent {
	input = strings.TrimSuffix(input, ".zip")
	parts := strings.Split(input, "_")
	if len(parts) == 1 {
		return ModIdent{input, nil}
	}

	name := strings.Join(parts[:len(parts)-1], "_")
	version, err := newVersion(parts[len(parts)-1])
	if err != nil {
		return ModIdent{input, nil}
	}
	return ModIdent{name, version}
}

func (i *ModIdent) toString() string {
	if i.Version != nil {
		return i.Name + " " + i.Version.toString(false)
	}
	return i.Name
}
