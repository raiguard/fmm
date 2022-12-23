package main

import "testing"

func TestVersion(t *testing.T) {
	tests := []struct {
		input           string
		output          string
		outputWithBuild string
		p0, p1, p2, p3  uint16
	}{
		{"1.0", "1.0.0", "1.0.0.0", 1, 0, 0, 0},
		{"1.1.15", "1.1.15", "1.1.15.0", 1, 1, 15, 0},
		{"2.3.4.5", "2.3.4", "2.3.4.5", 2, 3, 4, 5},
		{"010.001.0100.0000001", "10.1.100", "10.1.100.1", 10, 1, 100, 1},
	}
	for _, test := range tests {
		ver, err := newVersion(test.input)
		if err != nil {
			t.Error(err)
		}
		if ver[0] != test.p0 || ver[1] != test.p1 || ver[2] != test.p2 || ver[3] != test.p3 {
			t.Error("Version parse mismatch:", test.input, ver)
		}
		verStr := ver.toString(false)
		if verStr != test.output {
			t.Error("Version string mismatch:", test.input, verStr)
		}
		verStrWithBuild := ver.toString(true)
		if verStrWithBuild != test.outputWithBuild {
			t.Error("Version string mismatch:", test.input, verStr)
		}
	}
}

func TestVersionCmp(t *testing.T) {
	tests := []struct {
		v1  version
		v2  version
		res versionCmpRes
	}{
		{version{1, 3, 1}, version{2, 0}, versionLt},
		{version{1, 5, 3}, version{1, 5, 2}, versionGt},
		{version{1, 5}, version{1, 5, 0, 0}, versionEq},
	}
	for _, test := range tests {
		res := test.v1.cmp(test.v2)
		if res != test.res {
			t.Error("Version comparison failure:", test.v1, test.v2, test.res, res)
		}
	}
}
