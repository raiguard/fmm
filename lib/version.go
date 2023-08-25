package fmm

import (
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

type Version [4]uint16

type VersionCmpRes uint8

const (
	VersionAny VersionCmpRes = iota
	VersionEq
	VersionGt
	VersionGtEq
	VersionLt
	VersionLtEq
)

func NewVersion(input string) (*Version, error) {
	parts := strings.Split(strings.TrimSpace(input), ".")
	if len(parts) < 2 || len(parts) > 4 {
		return nil, errors.New("version string must have between 2 and 4 parts")
	}
	var ver Version
	for i, part := range parts {
		part, err := strconv.ParseUint(part, 10, 0)
		if err != nil {
			return nil, err
		}
		ver[i] = uint16(part)
	}
	return &ver, nil
}

func (v *Version) Cmp(other *Version) VersionCmpRes {
	if other == nil {
		return VersionEq
	}
	for i := range v {
		if v[i] > other[i] {
			return VersionGt
		} else if v[i] < other[i] {
			return VersionLt
		}
	}
	return VersionEq
}

func (v *Version) ToString(includeBuild bool) string {
	if includeBuild {
		return fmt.Sprintf("%d.%d.%d.%d", v[0], v[1], v[2], v[3])
	} else {
		return fmt.Sprintf("%d.%d.%d", v[0], v[1], v[2])
	}
}

func (v *Version) MarshalJSON() ([]byte, error) {
	return []byte("\"" + v.ToString(false) + "\""), nil
}

func (v *Version) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return err
	}

	ver, err := NewVersion(s)
	if err != nil {
		return err
	}

	v[0] = ver[0]
	v[1] = ver[1]
	v[2] = ver[2]
	v[3] = ver[3]

	return nil
}
