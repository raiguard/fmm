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

func download(args []string) {
	if len(args) == 0 {
		usage(downloadUsage, "no mods were provided")
	}

	if downloadToken == "" {
		abort("Download username not specified in config file")
	}
	if downloadToken == "" {
		abort("Download token not specified in config file")
	}

	dir, err := newDir(modsDir)
	if err != nil {
		abort(err)
	}

	var mods []Dependency
	for _, input := range args {
		mods = append(mods, Dependency{Ident: newModIdent(input), Req: VersionEq})
	}

	for _, mod := range mods {
		// TODO: Do we want to do this?
		if file, _, _ := dir.Find(mod); file != nil {
			fmt.Println(file.Ident.toString(), "is already in the mods directory")
			continue
		}

		err := downloadMod(mod, dir)
		if err != nil {
			errorln(err)
		}
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
		if entry.Enabled {
			if mod.Ident.Version == nil || entry.Version == nil {
				continue
			}
			if mod.Test(&ModIdent{entry.Name, entry.Version}) {
				continue
			}
		}
		entry.Enabled = true
		entry.Version = mod.Ident.Version
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

func upload(files []string) {
	if apiKey == "" {
		abort("Upload API key not specified in config file.")
	}
	for _, file := range files {
		if err := uploadMod(file); err != nil {
			abort("Upload failed:", err)
		}
	}
}
