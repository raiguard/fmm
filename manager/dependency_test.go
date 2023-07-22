package manager

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
		dep, err := NewDependency(test.input)
		if err != nil {
			t.Error(err)
		}
		assert.Equal(t, dep.Name, test.name)
		if test.version == nil {
			assert.Nil(t, dep.Version)
		} else {
			assert.NotNil(t, dep.Version)
			assert.Equal(t, dep.Version.Cmp(test.version), test.req)
		}

		assert.Equal(t, dep.Kind, test.kind)
		assert.Equal(t, dep.Req, test.req)
	}
}

func TestDependencyTest(t *testing.T) {
	tests := []struct {
		dep, name string
		version   Version
		result    bool
	}{
		{"flib", "flib", Version{0, 1, 1}, true},
		{"! flib", "flib", Version{0, 1, 1}, false},
		{"flib >= 0.10", "flib", Version{0, 1, 1}, false},
		{"flib >= 0.10", "flib", Version{0, 10, 0}, true},
		{"flib >= 0.10.0", "flib", Version{0, 10, 0}, true},
		{"flib > 0.10", "flib", Version{0, 10, 0}, false},
		{"flib>=0.10", "flib", Version{0, 10, 0}, true},
	}

	for _, test := range tests {
		dep, err := NewDependency(test.dep)
		assert.NoError(t, err)
		assert.Equal(t, dep.Test(&test.version), test.result)
	}
}
