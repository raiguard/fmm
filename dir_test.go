package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDir(t *testing.T) {
	dir := newDir("TEST/mods")

	// Check validity of mod structures
	assert.Equal(t, len(dir.Files), 3)

	expected := []ModIdent{
		{"Unzipped", &Version{1, 0, 0, 0}},
		{"UnzippedVersionless", &Version{1, 0, 0, 0}},
		{"Zipped", &Version{1, 1, 0, 0}},
	}

	for _, expected := range expected {
		file, err := dir.Find(Dependency{
			expected, DependencyRequired, VersionEq,
		})
		assert.NoError(t, err)
		assert.Equal(t, file.Ident.Name, expected.Name)
		assert.Equal(t, file.Ident.Version.cmp(expected.Version), VersionEq)
	}
}
