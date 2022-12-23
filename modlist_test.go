package main

import (
	"testing"
)

func TestModlistNew(t *testing.T) {
	_, err := newModlist("TEST/mods/mod-list.json")
	if err != nil {
		t.Error(err)
	}
}

func TestModlistOps(t *testing.T) {
	list, err := newModlist("TEST/mods/mod-list.json")
	if err != nil {
		t.Error(err)
	}
	list.disable("Unzipped")
	if list.isEnabled("Unzipped") {
		t.Error("Disable failed")
	}
	list.enable("Unzipped", nil)
	if !list.isEnabled("Unzipped") {
		t.Error("Enable failed")
	}
}
