package fmm

import (
	"bytes"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestModSettings(t *testing.T) {
	origBytes, err := os.ReadFile("../TEST/mods/mod-settings.dat")
	require.NoError(t, err)
	r := newDatReader(bytes.NewReader(origBytes))
	settings := r.ReadModSettings()
	b := bytes.Buffer{}
	w := newDatWriter(&b)
	w.WriteModSettings(&settings)
	w.writer.Flush()
	newBytes := b.Bytes()
	require.Equal(t, len(origBytes), len(newBytes))
	// We can't check specific contents because dictionaries may not be written in the same order they were read.
}
