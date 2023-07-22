package cli

import (
	"strings"

	fmm "github.com/raiguard/fmm/manager"
)

type ModIdent struct {
	Name    string
	Version *fmm.Version
}

func NewModIdent(input string) ModIdent {
	input = strings.TrimSuffix(input, ".zip")
	parts := strings.Split(input, "_")
	if len(parts) == 1 {
		return ModIdent{input, nil}
	}

	name := strings.Join(parts[:len(parts)-1], "_")
	version, err := fmm.NewVersion(parts[len(parts)-1])
	if err != nil {
		return ModIdent{input, nil}
	}
	return ModIdent{name, version}
}

func (i *ModIdent) ToString() string {
	if i.Version != nil {
		return i.Name + " " + i.Version.ToString(false)
	}
	return i.Name
}