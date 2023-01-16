package main

import (
	"fmt"
	"path"
)

func disable(args []string) {
	if len(args) == 0 {
		disableAll()
		return
	}

	dir, err := newDir(modsDir)
	if err != nil {
		abort(err)
	}
	defer dir.save()

	var mods []Dependency
	for _, input := range args {
		mods = append(mods, Dependency{Ident: newModIdent(input), Req: VersionAny})
	}
	for _, mod := range mods {
		file, entry, err := dir.Find(mod)
		if err != nil {
			errorln(err)
			continue
		}
		if !entry.Enabled {
			continue
		}
		entry.Enabled = false
		fmt.Println("Disabled", file.Ident.Name)
	}
}

func disableAll() {
	list, err := newModList(path.Join(modsDir, "mod-list.json"))
	if err != nil {
		usage(disableUsage, err)
	}
	defer list.save()

	for i := range list.Mods {
		mod := &list.Mods[i]
		if mod.Name != "base" {
			mod.Enabled = false
		}
	}

	fmt.Println("Disabled all mods")
}

func enable(args []string) {
	if len(args) == 0 {
		usage(enableUsage, "no mods were provided")
	}

	dir, err := newDir(modsDir)
	if err != nil {
		abort(err)
	}
	defer dir.save()

	var mods []Dependency
	for _, input := range args {
		mods = append(mods, Dependency{Ident: newModIdent(input), Req: VersionEq})
	}

	i := 0
	for {
		if i > len(mods)-1 {
			break
		}
		mod := mods[i]
		i++
		file, entry, err := dir.Find(mod)
		if err != nil {
			errorln(err)
			continue
		}
		// TODO: This prevents enabling a different version of the same mod
		if entry.Enabled {
			continue
		}
		entry.Enabled = true
		if mod.Ident.Version != nil {
			versionStr := mod.Ident.Version.toString(false)
			entry.Version = &versionStr
		}
		fmt.Println("Enabled", file.Ident.toString())

		deps, err := file.Dependencies()
		if err != nil {
			errorln(err)
		}
		if deps == nil {
			continue
		}

		for _, dep := range *deps {
			if dep.Ident.Name != "base" && dep.Kind == DependencyRequired {
				mods = append(mods, dep)
			}
		}
	}
}
