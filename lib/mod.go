package fmm

type Mod struct {
	Name       string
	Enabled    *Version
	releases   []*Release
	isInternal bool
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
