package manager

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestManager(t *testing.T) {
	manager, err := NewManager("../TEST")
	require.NoError(t, err)
	require.Equal(t, len(manager.mods), 3)

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
		require.NoError(t, err)
		release := mod.GetLatestRelease()
		require.NotNil(t, release)
		require.Equal(t, release.Name, expected.name)
		require.Equal(t, release.Version.Cmp(&expected.version), VersionEq)
	}
}
