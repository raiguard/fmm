package main

import (
	"encoding/json"
	"strings"
)

type Dependency struct {
	Ident ModIdent
	Kind  DependencyKind
	Req   VersionCmpRes
}

type DependencyKind uint8

const (
	DependencyHiddenOptional DependencyKind = iota
	DependencyIncompatible
	DependencyNoLoadOrder
	DependencyOptional
	DependencyRequired
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

	// Iterate in reverse and find the first space
	var ver *Version = nil
	for i := len(input) - 1; i >= 0; i-- {
		if i > 0 && !(input[i] == '.' || (input[i] >= '0' && input[i] <= '9')) {
			parsed, err := newVersion(input[i:])
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

	return &Dependency{
		Ident: ModIdent{name, ver},
		Kind:  kind,
		Req:   req,
	}, nil
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

	d.Ident = dep.Ident
	d.Kind = dep.Kind
	d.Req = dep.Req

	return nil
}
