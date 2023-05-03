package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

type ModIdentAndPresence struct {
	Ident     ModIdent
	IsPresent bool
}

func parseCliInput(input []string, parseDependencies bool) []ModIdentAndPresence {
	var mods []ModIdent

	for _, input := range input {
		if strings.HasSuffix(input, ".zip") {
			// TODO: Read from save
		} else if strings.HasSuffix(input, ".log") {
			mods = append(mods, parseLogFile(input)...)
		} else if strings.HasSuffix(input, ".json") {
			// TODO: mod-list.json
		} else if strings.HasPrefix(input, "!") {
			// TODO: Mod set
		} else {
			mods = append(mods, newModIdent(input))
		}
	}

	if parseDependencies {
		mods = expandDependencies(mods)
	}

	var output []ModIdentAndPresence

	dir := newDir(modsDir)

	for _, mod := range mods {
		present := dir.Find(Dependency{mod, DependencyRequired, VersionAny}) != nil
		output = append(output, ModIdentAndPresence{mod, present})
	}

	return output
}

func expandDependencies(mods []ModIdent) []ModIdent {
	visited := make(map[string]bool)

	dir := newDir(modsDir)

	for i := 0; i < len(mods); i += 1 {
		mod := mods[i]
		visited[mod.Name] = true
		file := dir.Find(Dependency{mod, DependencyRequired, VersionAny})
		var deps []Dependency
		var err error
		if file != nil {
			realDeps, err := file.Dependencies()
			if err != nil {
				errorln(err)
				continue
			}
			deps = *realDeps
		} else {
			deps, err = portalGetDependencies(mod)
			if err != nil {
				errorln(err)
				continue
			}
		}
		for _, dep := range deps {
			// FIXME: Dependency kind or version might be different / incompatible
			if dep.Ident.Name == "base" || visited[dep.Ident.Name] {
				continue
			}
			if dep.Kind == DependencyRequired || dep.Kind == DependencyNoLoadOrder {
				visited[dep.Ident.Name] = true
				mods = append(mods, dep.Ident)
			}
		}
	}

	return mods
}

func parseLogFile(filepath string) []ModIdent {
	var output []ModIdent
	file, err := os.Open(filepath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading %s: %s", filepath, err)
		return output
	}
	defer file.Close()

	fileScanner := bufio.NewScanner(file)
	fileScanner.Split(bufio.ScanLines)

	inChecksums := false
	for fileScanner.Scan() {
		line := fileScanner.Text()
		if !strings.Contains(line, "Checksum of") {
			if inChecksums {
				break
			} else {
				continue
			}
		}
		inChecksums = true
		parts := strings.Split(strings.TrimSpace(line), " ")
		modName, _ := strings.CutSuffix(strings.Join(parts[3:len(parts)-1], " "), ":")
		if modName == "base" {
			continue
		}
		output = append(output, ModIdent{modName, nil})
	}

	return output
}
