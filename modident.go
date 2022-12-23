package main

import (
	"strings"
)

type modident struct {
	Name    string
	Version *version
}

func newModident(input string) modident {
	input = strings.TrimSuffix(input, ".zip")
	parts := strings.Split(input, "_")
	if len(parts) == 1 {
		return modident{input, nil}
	}

	name := strings.Join(parts[:len(parts)-1], "_")
	version, _ := newVersion(parts[len(parts)-1])
	return modident{name, version}
}

func (i *modident) toString() string {
	if i.Version != nil {
		return i.Name + "_" + i.Version.toString(false)
	}
	return i.Name
}
