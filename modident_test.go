package main

import "testing"

func TestModident(t *testing.T) {
	tests := []struct {
		input    string
		expected ModIdent
	}{
		{"Zipped", ModIdent{"Zipped", nil}},
		{"Zipped 1.0.0", ModIdent{"Zipped", &Version{1}}},
		{"Recipe_Book 1.0.35", ModIdent{"Recipe_Book", &Version{1, 0, 35}}},
	}
	for _, test := range tests {
		mod := newModIdent(test.input)
		if mod.Name != test.expected.Name {
			t.Error("Mod name mismatch:", test.input, mod, test.expected)
		}
		if test.expected.Version != nil {
			if mod.Version == nil || test.expected.Version.cmp(*mod.Version) != VersionEq {
				t.Error("Mod version mismatch:", test.input, mod, test.expected)
			}
		} else if mod.Version != nil {
			t.Error("Mod version mismatch:", test.input, mod, test.expected)
		}
		modStr := mod.toString()
		if modStr != test.input {
			t.Error("Mod string mismatch:", test.input, modStr)
		}
	}
}
