package fmm

import (
	"encoding/json"
	"errors"
	"os"
)

type ModListJson struct {
	Mods []ModListJsonMod `json:"mods"`
}

type ModListJsonMod struct {
	Name    string   `json:"name"`
	Enabled bool     `json:"enabled"`
	Version *Version `json:"version,omitempty"`
	// TODO: Remove this in Go 1.21
	isInternal bool
}

func ParseModListJson(path string) (*ModListJson, error) {
	modListJsonData, err := os.ReadFile(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, nil
		}
		return nil, errors.Join(errors.New("error reading mod-list.json"), err)
	}

	var modListJson ModListJson
	if err = json.Unmarshal(modListJsonData, &modListJson); err != nil {
		return nil, errors.Join(errors.New("error parsing mod-list.json"), err)
	}
	return &modListJson, nil
}
