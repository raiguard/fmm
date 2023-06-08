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

	list := newModList(path.Join(modsDir, "mod-list.json"))
	defer list.Save()

	mods := parseCliInput(args, false)
	for _, mod := range mods {
		list.Disable(mod.Ident.Name)
	}
}

func disableAll() {
	list := newModList(path.Join(modsDir, "mod-list.json"))
	defer list.Save()

	for i := range list.Mods {
		mod := &list.Mods[i]
		if mod.Name != "base" {
			mod.Enabled = false
		}
	}

	fmt.Println("Disabled all mods")
}

func enable(args []string) {
	mods := parseCliInput(args, true)

	list := newModList(path.Join(modsDir, "mod-list.json"))
	defer list.Save()

	for i := 0; i < len(mods); i += 1 {
		mod := mods[i]
		if !mod.IsPresent {
			err := portalDownloadMod(Dependency{mod.Ident, DependencyRequired, VersionEq})
			if err != nil {
				errorln(err)
				continue
			}
		}
		list.Enable(mod.Ident)
	}
}

func list(args []string) {
	if len(args) == 0 {
		dir := newDir(modsDir)

		for _, file := range dir {
			// We don't use toString() here because we want the underscore
			output := file.Ident.Name + "_" + file.Ident.Version.toString(false)
			fmt.Println(output)
		}
	}

	mods := parseCliInput(args, false)
	for _, mod := range mods {
		fmt.Println(mod.Ident.toString())
	}
}

func sync(args []string) {
	disableAll()
	enable(args)
}

func upload(files []string) {
	if apiKey == "" {
		abort("API key not specified.")
	}
	if len(files) == 0 {
		abort("no files were provided")
	}
	for _, file := range files {
		if err := portalUploadMod(file); err != nil {
			abort("Upload failed:", err)
		}
	}
}
