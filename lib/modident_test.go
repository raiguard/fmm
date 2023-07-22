package fmm

import (
	"testing"

	"github.com/stretchr/testify/require"
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
		mod := NewModIdent(test.input)
		require.Equal(t, mod.Name, test.expected.Name)
		if test.expected.Version != nil {
			require.NotNil(t, mod.Version)
			require.Equal(t, test.expected.Version.Cmp(mod.Version), VersionEq)
		} else {
			require.Nil(t, mod.Version)
		}
		require.Equal(t, mod.ToString(), test.output)
	}
}
