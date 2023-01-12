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
	version, _ := newVersion(parts[len(parts)-1])
	return ModIdent{name, version}
}

func (i *ModIdent) toString() string {
	if i.Version != nil {
		return i.Name + "_" + i.Version.toString(false)
	}
	return i.Name
}
