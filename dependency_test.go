package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewDependency(t *testing.T) {
	tests := []struct {
		input   string
		name    string
		version *Version
		kind    DependencyKind
		req     VersionCmpRes
	}{
		{"flib", "flib", nil, DependencyRequired, VersionAny},
	}
	for _, test := range tests {
		dep, err := newDependency(test.input)
		if err != nil {
			t.Error(err)
		}
		assert.Equal(t, dep.Ident.Name, test.name)
		if test.version == nil {
			assert.Nil(t, dep.Ident.Version)
		} else {
			assert.NotNil(t, dep.Ident.Version)
			assert.Equal(t, dep.Ident.Version.cmp(test.version), test.req)
		}

		assert.Equal(t, dep.Kind, test.kind)
		assert.Equal(t, dep.Req, test.req)
	}
}

func TestDependencyTest(t *testing.T) {
	tests := []struct {
		dep, mod string
		result   bool
	}{
		{"flib", "flib_0.1.1", true},
		{"! flib", "flib_0.1.1", false},
		{"flib >= 0.10", "flib_0.1.1", false},
		{"flib >= 0.10", "flib_0.10.0", true},
		{"flib >= 0.10.0", "flib_0.10.0", true},
		{"flib > 0.10", "flib_0.10.0", false},
	}

	for _, test := range tests {
		dep, err := newDependency(test.dep)
		assert.NoError(t, err)
		mod := newModIdent(test.mod)
		assert.Equal(t, dep.Test(&mod), test.result)
	}
}
