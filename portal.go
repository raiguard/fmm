package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"

	"github.com/cheggaaa/pb/v3"
)

const barTemplate string = `Downloading {{ string . "name" }} {{ bar . "[" "#" "#" " " "]" }} {{ counters . }} {{ percent . "%.0f%%" }}`

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

	// Check releases from newest to oldest and find the first matching one
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

	downloadUrl := fmt.Sprintf("https://mods.factorio.com/%s?username=%s&token=%s",
		release.DownloadUrl, downloadUsername, downloadToken)
	outPath := path.Join(modsDir, release.FileName)

	resp, err := http.Get(downloadUrl)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	f, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer f.Close()

	bar := pb.New64(resp.ContentLength)
	bar.SetTemplateString(barTemplate)
	bar.Set(pb.Bytes, true).Set("name", release.FileName)
	bar.Start()

	barReader := bar.NewProxyReader(resp.Body)
	io.Copy(f, barReader)

	bar.Finish()

	// TODO: Add to dir

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
