package main

import (
	"testing"
)

func TestModlistNew(t *testing.T) {
	_, err := newModList("TEST/mods/mod-list.json")
	if err != nil {
		t.Error(err)
	}
}

func TestModlistOps(t *testing.T) {
	list, err := newModList("TEST/mods/mod-list.json")
	if err != nil {
		t.Error(err)
	}
	list.Disable("Unzipped")
	if list.IsEnabled("Unzipped") {
		t.Error("Disable failed")
	}
	list.Enable(ModIdent{"Unzipped", nil})
	if !list.IsEnabled("Unzipped") {
		t.Error("Enable failed")
	}
}
