package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"path"

	"github.com/cavaliergopher/grab/v3"
)

func downloadMod(mod Dependency, dir *Dir) error {
	url := fmt.Sprintf("https://mods.factorio.com/api/mods/%s", mod.Ident.Name)
	res, err := http.Get(url)
	if err != nil {
		return err
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return err
	}
	res.Body.Close()

	var unmarshaled ModRes
	err = json.Unmarshal(body, &unmarshaled)
	if err != nil {
		return err
	}

	var release *ModResRelease
	for i := len(unmarshaled.Releases) - 1; i >= 0; i -= 1 {
		toCheck := unmarshaled.Releases[i]
		if mod.Test(&ModIdent{mod.Ident.Name, &toCheck.Version}) {
			release = &toCheck
			break
		}
	}

	if release == nil {
		return errors.New(fmt.Sprintf("%s was not found on the mod portal",
			mod.Ident.toString()))
	}

	fmt.Printf("Downloading %s %s\n", unmarshaled.Name, release.Version.toString(false))

	downloadUrl := fmt.Sprintf("https://mods.factorio.com/%s?username=%s&token=%s",
		release.DownloadUrl, downloadUsername, downloadToken)
	outPath := path.Join(modsDir, release.FileName)
	resp, err := grab.Get(outPath, downloadUrl)
	if err != nil {
		return err
	}

	fmt.Printf("Downloaded to %s\n", resp.Filename)

	// TODO: Add to dir and download dependencies

	return nil
}

type ModRes struct {
	Name     string
	Releases []ModResRelease
	Title    string
}

type ModResRelease struct {
	DownloadUrl string `json:"download_url"`
	FileName    string `json:"file_name"`
	Version     Version
}
