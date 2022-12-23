package main

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
)

type version [4]uint16

type versionCmpRes uint8

const (
	versionAny versionCmpRes = iota
	versionEq
	versionGt
	versionGtEq
	versionLt
	versionLtEq
)

func newVersion(input string) (*version, error) {
	parts := strings.Split(strings.TrimSpace(input), ".")
	if len(parts) < 2 || len(parts) > 4 {
		return nil, errors.New("Version string must have between 2 and 4 parts")
	}
	var ver version
	for i, part := range parts {
		part, err := strconv.ParseUint(part, 10, 0)
		if err != nil {
			return nil, err
		}
		ver[i] = uint16(part)
	}
	return &ver, nil
}

func (v *version) cmp(other version) versionCmpRes {
	for i := range v {
		if v[i] > other[i] {
			return versionGt
		} else if v[i] < other[i] {
			return versionLt
		}
	}
	return versionEq
}

func (v *version) toString(includeBuild bool) string {
	if includeBuild {
		return fmt.Sprintf("%d.%d.%d.%d", v[0], v[1], v[2], v[3])
	} else {
		return fmt.Sprintf("%d.%d.%d", v[0], v[1], v[2])
	}
}
