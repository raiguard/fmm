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

	mods := newModIdentList(args)
	for _, mod := range mods {
		file, entry, err := dir.find(mod)
		if err != nil {
			errorln(err)
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

	mods := newModIdentList(args)
	for _, mod := range mods {
		file, entry, err := dir.find(mod)
		if err != nil {
			errorln(err)
			continue
		}
		entry.Enabled = true
		if mod.Version != nil {
			versionStr := mod.Version.toString(false)
			entry.Version = &versionStr
		}
		fmt.Println("Enabled", file.Ident.toString())
	}
}
