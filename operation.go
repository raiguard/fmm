package main

import (
	"fmt"
	"path"
)

func disable(args []string) {
	list, err := newModList(path.Join(modsDir, "mod-list.json"))
	if err != nil {
		usage(disableUsage, err)
	}
	defer list.save()

	mods := newModIdentList(args)

	// Disable all
	if len(mods) == 0 {
		for i := range list.Mods {
			mod := &list.Mods[i]
			if mod.Name != "base" {
				mod.Enabled = false
			}
		}
		fmt.Println("Disabled all mods")
		return
	}

	for _, mod := range mods {
		list.disable(mod.Name)
		fmt.Println("Disabled", mod.toString())
	}
}

func enable(args []string) {
	if len(args) == 0 {
		usage(enableUsage, "no mods were provided")
	}
	list, err := newModList(path.Join(modsDir, "mod-list.json"))
	if err != nil {
		usage(enableUsage, err)
	}
	defer list.save()

	mods := newModIdentList(args)
	for _, mod := range mods {
		list.enable(mod.Name, mod.Version)
		fmt.Println("Enabled", mod.toString())
	}
}
