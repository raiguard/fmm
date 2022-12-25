package main

import "testing"

func TestModident(t *testing.T) {
	tests := []struct {
		input    string
		expected Modident
	}{
		{"Zipped", Modident{"Zipped", nil}},
		{"Zipped_1.0.0", Modident{"Zipped", &version{1}}},
		{"Recipe_Book_1.0.35", Modident{"Recipe_Book", &version{1, 0, 35}}},
	}
	for _, test := range tests {
		mod := newModident(test.input)
		if mod.Name != test.expected.Name {
			t.Error("Mod name mismatch:", test.input, mod, test.expected)
		}
		if test.expected.Version != nil {
			if mod.Version == nil || test.expected.Version.cmp(*mod.Version) != versionEq {
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
