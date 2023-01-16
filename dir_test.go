package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDir(t *testing.T) {
	dir, err := newDir("TEST/mods")
	assert.NoError(t, err)

	// Check validity of mod structures
	assert.Equal(t, len(dir.Files), 3)
	assert.Equal(t, len(dir.List.Mods), 4)

	expected := []ModIdent{
		{"Unzipped", &Version{1, 0, 0, 0}},
		{"UnzippedVersionless", &Version{1, 0, 0, 0}},
		{"Zipped", &Version{1, 1, 0, 0}},
	}

	for _, expected := range expected {
		file, entry, err := dir.Find(expected, VersionEq)
		assert.NoError(t, err)
		assert.Equal(t, file.Ident.Name, expected.Name)
		assert.Equal(t, file.Ident.Version.cmp(*expected.Version), VersionEq)
		assert.Equal(t, entry.Name, expected.Name)
	}
}
