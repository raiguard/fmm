package manager

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDir(t *testing.T) {
	dir := newDir("TEST/mods")

	// Check validity of mod structures
	assert.Equal(t, len(dir), 3)

	expected := []ModIdent{
		{"Unzipped", &Version{1, 0, 0, 0}},
		{"UnzippedVersionless", &Version{1, 0, 0, 0}},
		{"Zipped", &Version{1, 1, 0, 0}},
	}

	for _, expected := range expected {
		file := dir.Find(Dependency{
			expected, DependencyRequired, VersionEq,
		})
		assert.Equal(t, file.Ident.Name, expected.Name)
		assert.Equal(t, file.Ident.Version.cmp(expected.Version), VersionEq)
	}
}
