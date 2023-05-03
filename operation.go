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

	mods := parseMods(args, false)
	for _, mod := range mods {
		list.Disable(mod.Name)
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
	if len(args) == 0 {
		abort("no mods were provided")
	}

	mods := parseMods(args, true)

	list := newModList(path.Join(modsDir, "mod-list.json"))
	defer list.Save()

	for i := 0; i < len(mods); i += 1 {
		mod := mods[i]
		list.Enable(mod)
	}
}

// func install(args []string) {
// 	if len(args) == 0 {
// 		abort("no mods were provided")
// 	}

// 	if downloadUsername == "" {
// 		abort("Username not specified")
// 	}
// 	if downloadToken == "" {
// 		abort("Token not specified")
// 	}

// 	dir := newDir(modsDir)

// 	var mods []Dependency
// 	for _, input := range args {
// 		mods = append(mods, Dependency{Ident: newModIdent(input), Req: VersionEq})
// 	}

// 	for _, mod := range mods {
// 		// TODO: Do we want to do this?
// 		if file, _ := dir.Find(mod); file != nil {
// 			fmt.Println(file.Ident.toString(), "is already in the mods directory")
// 			continue
// 		}

// 		err := portalDownloadMod(mod, dir)
// 		if err != nil {
// 			errorln(err)
// 		}
// 	}
// }

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
