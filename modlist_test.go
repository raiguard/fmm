package main

import (
	"testing"
)

func TestModlistNew(t *testing.T) {
	newModList("TEST/mods/mod-list.json")
}

func TestModlistOps(t *testing.T) {
	list := newModList("TEST/mods/mod-list.json")
	list.Disable("Unzipped")
	if list.IsEnabled("Unzipped") {
		t.Error("Disable failed")
	}
	list.Enable(ModIdent{"Unzipped", nil})
	if !list.IsEnabled("Unzipped") {
		t.Error("Enable failed")
	}
}
