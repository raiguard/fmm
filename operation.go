package main

import (
	"fmt"
)

func disable(manager *Manager, args []string) {
	if len(args) == 0 {
		manager.DisableAll()
		fmt.Println("Disabled all mods")
		return
	}

	mods := parseCliInput(args, false)
	for _, mod := range mods {
		manager.Disable(mod.Name)
		fmt.Println("Disabled", mod.Name)
	}
}

func enable(manager *Manager, args []string) {
	mods := parseCliInput(args, true)

	for _, mod := range mods {
		// if !mod.IsPresent {
		// 	err := portalDownloadMod(Dependency{mod.Ident, DependencyRequired, VersionEq})
		// 	if err != nil {
		// 		errorln(err)
		// 		continue
		// 	}
		// }
		if err := manager.Enable(mod); err != nil {
			errorln(err)
		} else {
			fmt.Println("Enabled", mod.toString())
		}
	}
}

func list(manager *Manager, args []string) {
	// if len(args) == 0 {
	// 	dir := newDir(manager.modsDir)

	// 	for _, file := range dir {
	// 		// We don't use toString() here because we want the underscore
	// 		output := file.Ident.Name + "_" + file.Ident.Version.toString(false)
	// 		fmt.Println(output)
	// 	}
	// }

	// mods := parseCliInput(args, false)
	// for _, mod := range mods {
	// 	fmt.Println(mod.Ident.toString())
	// }
}

func sync(manager *Manager, args []string) {
	// manager.disableAll()
	// manager.enable(args)
}

func upload(manager *Manager, files []string) {
	// if apiKey == "" {
	// 	abort("API key not specified.")
	// }
	// if len(files) == 0 {
	// 	abort("no files were provided")
	// }
	// for _, file := range files {
	// 	if err := portalUploadMod(file); err != nil {
	// 		abort("Upload failed:", err)
	// 	}
	// }
}
