package cli

import (
	"strings"

	fmm "github.com/raiguard/fmm/lib"
)

func parseCliInput(input []string, parseDependencies bool) []fmm.ModIdent {
	var mods []fmm.ModIdent

	for _, input := range input {
		var thisMods []fmm.ModIdent
		var err error
		if strings.HasSuffix(input, ".zip") {
			thisMods, err = fmm.ParseSaveFile(input)
		} else if strings.HasSuffix(input, ".log") {
			thisMods = fmm.ParseLogFile(input)
		} else if strings.HasSuffix(input, ".json") {
			// TODO: mod-list.json
		} else if strings.HasPrefix(input, "!") {
			// TODO: Mod set
		} else {
			thisMods = append(thisMods, fmm.NewModIdent(input))
		}
		if err != nil {
			errorln(err)
			continue
		}
		mods = append(mods, thisMods...)
	}

	return mods
}

// func expandDependencies(manager *Manager, mods []ModIdent) []ModIdent {
// 	visited := make(map[string]bool)
// 	toVisit := []Dependency{}
// 	for _, mod := range mods {
// 		toVisit = append(toVisit, Dependency{mod, DependencyRequired, VersionEq})
// 	}
// 	output := []ModIdent{}

// 	for i := 0; i < len(toVisit); i += 1 {
// 		mod := toVisit[i]
// 		if _, exists := visited[mod.Ident.Name]; exists {
// 			continue
// 		}
// 		visited[mod.Ident.Name] = true
// 		var ident ModIdent
// 		var deps []Dependency
// 		var err error
// 		if file := manager.Find(mod); file != nil {
// 			ident = file.Ident
// 			deps, err = file.Dependencies()
// 		} else if mod.Ident.Name == "base" {
// 			// TODO: Check against dependency constraint?
// 			ident = mod.Ident
// 		} else {
// 			var release *PortalModRelease
// 			release, err = portalGetRelease(mod)
// 			if err == nil {
// 				ident = ModIdent{mod.Ident.Name, &release.Version}
// 				deps = release.InfoJson.Dependencies
// 			}
// 		}
// 		if err != nil {
// 			errorln(err)
// 			continue
// 		}
// 		output = append(output, ident)
// 		for _, dep := range deps {
// 			if dep.Ident.Name == "base" {
// 				continue
// 			}
// 			if dep.Kind == DependencyRequired || dep.Kind == DependencyNoLoadOrder {
// 				toVisit = append(toVisit, dep)
// 			}
// 		}
// 	}

// 	return output
// }
