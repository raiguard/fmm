package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestModIdent(t *testing.T) {
	tests := []struct {
		input, output string
		expected      ModIdent
	}{
		{"Zipped", "Zipped", ModIdent{"Zipped", nil}},
		{"Zipped_1.0.0", "Zipped 1.0.0", ModIdent{"Zipped", &Version{1}}},
		{"Recipe_Book_1.0.35.zip", "Recipe_Book 1.0.35", ModIdent{"Recipe_Book", &Version{1, 0, 35}}},
	}
	for _, test := range tests {
		mod := newModIdent(test.input)
		assert.Equal(t, mod.Name, test.expected.Name)
		if test.expected.Version != nil {
			assert.NotNil(t, mod.Version)
			assert.Equal(t, test.expected.Version.cmp(*mod.Version), VersionEq)
		} else {
			assert.Nil(t, mod.Version)
		}
		assert.Equal(t, mod.toString(), test.output)
	}
}
