package main

import (
	"strings"
)

type Modident struct {
	Name    string
	Version *version
}

func newModident(input string) Modident {
	input = strings.TrimSuffix(input, ".zip")
	parts := strings.Split(input, "_")
	if len(parts) == 1 {
		return Modident{input, nil}
	}

	name := strings.Join(parts[:len(parts)-1], "_")
	version, _ := newVersion(parts[len(parts)-1])
	return Modident{name, version}
}

func (i *Modident) toString() string {
	if i.Version != nil {
		return i.Name + "_" + i.Version.toString(false)
	}
	return i.Name
}
