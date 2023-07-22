package manager

import (
	"encoding/json"
	"strings"
)

type Dependency struct {
	Name    string
	Version *Version
	Kind    DependencyKind
	Req     VersionCmpRes
}

type DependencyKind uint8

const (
	DependencyRequired DependencyKind = iota
	DependencyOptional
	DependencyHiddenOptional
	DependencyIncompatible
	DependencyNoLoadOrder
)

func newDependency(input string) (*Dependency, error) {
	input = strings.TrimSpace(input)

	kind := DependencyRequired
	if strings.HasPrefix(input, "!") {
		kind = DependencyIncompatible
		input = strings.TrimPrefix(input, "!")
	} else if strings.HasPrefix(input, "?") {
		kind = DependencyOptional
		input = strings.TrimPrefix(input, "?")
	} else if strings.HasPrefix(input, "(?)") {
		kind = DependencyHiddenOptional
		input = strings.TrimPrefix(input, "(?)")
	} else if strings.HasPrefix(input, "~") {
		kind = DependencyNoLoadOrder
		input = strings.TrimPrefix(input, "~")
	}

	// Iterate in reverse and find the first non-digit and non-dot
	var ver *Version
	for i := len(input) - 1; i >= 0; i-- {
		if i > 0 && !(input[i] == '.' || (input[i] >= '0' && input[i] <= '9')) {
			parsed, err := NewVersion(input[i:])
			if err == nil {
				ver = parsed
				input = strings.TrimSpace(input[:i])
			}
			break
		}
	}

	req := VersionAny
	if strings.HasSuffix(input, "<=") {
		req = VersionLtEq
		input = strings.TrimSuffix(input, "<=")
	} else if strings.Contains(input, "<") {
		req = VersionLt
		input = strings.TrimSuffix(input, "<")
	} else if strings.Contains(input, ">=") {
		req = VersionGtEq
		input = strings.TrimSuffix(input, ">=")
	} else if strings.Contains(input, ">") {
		req = VersionGt
		input = strings.TrimSuffix(input, ">")
	} else if strings.Contains(input, "=") {
		req = VersionEq
		input = strings.TrimSuffix(input, "=")
	}

	name := strings.TrimSpace(input)

	return &Dependency{name, ver, kind, req}, nil
}

func (d *Dependency) Test(ver *Version) bool {
	if ver == nil {
		return true
	}

	if d.Kind == DependencyIncompatible {
		return false
	}

	if d.Req == VersionAny {
		return true
	}

	return d.Req&ver.Cmp(d.Version) > 0
}

func (d *Dependency) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return err
	}

	dep, err := newDependency(s)
	if err != nil {
		return err
	}

	d.Name = dep.Name
	d.Version = dep.Version
	d.Kind = dep.Kind
	d.Req = dep.Req

	return nil
}
