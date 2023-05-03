package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

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
		modName, _ := strings.CutSuffix(strings.Split(strings.TrimSpace(line), " ")[3], ":")
		if modName == "base" {
			continue
		}
		output = append(output, ModIdent{modName, nil})
	}

	return output
}

func parseMods(input []string, parseDependencies bool) []ModIdent {
	var output []ModIdent

	for _, input := range input {
		if strings.HasSuffix(input, ".zip") {
			// TODO: Read from save
		} else if strings.HasSuffix(input, ".log") {
			output = append(output, parseLogFile(input)...)
		} else if strings.HasSuffix(input, ".json") {
			// TODO: mod-list.json
		} else if strings.HasPrefix(input, "!") {
			// TODO: Mod set
		} else {
			output = append(output, newModIdent(input))
		}
	}

	if parseDependencies {
		output = expandDependencies(output)
	}

	return output
}
