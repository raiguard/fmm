package manager

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestManager(t *testing.T) {
	manager, err := NewManager("../TEST")
	assert.NoError(t, err)
	assert.Equal(t, len(manager.mods), 3)

	expected := []struct {
		name    string
		version Version
	}{
		{"Unzipped", Version{1, 0, 0, 0}},
		{"UnzippedVersionless", Version{1, 0, 0, 0}},
		{"Zipped", Version{1, 1, 0, 0}},
	}

	for _, expected := range expected {
		mod, err := manager.GetMod(expected.name)
		assert.NoError(t, err)
		release := mod.GetLatestRelease()
		assert.NotNil(t, release)
		assert.Equal(t, release.Name, expected.name)
		assert.Equal(t, release.Version.Cmp(&expected.version), VersionEq)
	}
}
