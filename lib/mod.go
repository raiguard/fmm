package fmm

type Mod struct {
	Enabled  *Version
	Name     string
	releases Releases
}

func (m *Mod) GetLatestRelease() *Release {
	return m.releases[len(m.releases)-1]
}

func (m *Mod) GetRelease(version *Version) *Release {
	if version == nil {
		return m.GetLatestRelease()
	}
	return m.GetMatchingRelease(&Dependency{
		m.Name,
		version,
		DependencyRequired,
		VersionEq,
	})
}

func (m *Mod) GetMatchingRelease(dep *Dependency) *Release {
	// Iterate in reverse to get the newest version first
	for i := len(m.releases) - 1; i >= 0; i-- {
		release := m.releases[i]
		if dep.Test(&release.Version) {
			return release
		}
	}
	return nil
}

type Releases []*Release

// Implementations for sorting interface
// TODO: Use Go 1.21 `slices` module once it is released
func (r Releases) Len() int {
	return len(r)
}
func (r Releases) Swap(i, j int) {
	r[i], r[j] = r[j], r[i]
}
func (r Releases) Less(i, j int) bool {
	releaseI, releaseJ := r[i], r[j]
	return releaseI.Version.Cmp(&releaseJ.Version) == VersionLt
}
