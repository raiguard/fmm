package main

type mods []Mod

func (dir mods) Find(mod Dependency) *Mod {
	// Iterate in reverse to get the newest version first
	for i := len(dir) - 1; i >= 0; i-- {
		thisfile := &dir[i]
		if thisfile.Ident.Name != mod.Ident.Name {
			continue
		}
		if mod.Test(thisfile.Ident.Version) {
			return thisfile
		}
	}
	return nil
}

// Implementations for sorting interface
func (dir mods) Len() int {
	return len(dir)
}
func (dir mods) Swap(i, j int) {
	dir[i], dir[j] = dir[j], dir[i]
}
func (dir mods) Less(i, j int) bool {
	modi, modj := dir[i].Ident, &dir[j].Ident
	if modi.Name != modj.Name {
		return modi.Name < modj.Name
	}
	return modi.Version.cmp(modj.Version) == VersionLt
}
