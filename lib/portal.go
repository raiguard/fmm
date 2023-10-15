package fmm

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/url"
	"os"
	"path"

	"github.com/cavaliergopher/grab/v3"
)

type ModPortal struct {
	apiKey       string
	baseVersion  *Version
	downloadPath string
	mods         map[string]*PortalModInfo
	playerData   PlayerData
	server       string
}

// GetModInfo fetches information for the given mod from the mod portal.
func (p *ModPortal) GetModInfo(name string) (*PortalModInfo, error) {
	if mod := p.mods[name]; mod != nil {
		return mod, nil
	}

	fmt.Println("fetching info for", name) // TODO: Relocate this
	url, err := url.JoinPath(p.server, "api/mods", name, "full")
	if err != nil {
		return nil, err
	}
	res, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	if res.StatusCode != http.StatusOK {
		return nil, errors.New(fmt.Sprintf("%s was not found on the mod portal", name))
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}
	res.Body.Close()

	var mod PortalModInfo
	err = json.Unmarshal(body, &mod)
	if err != nil {
		return nil, err
	}

	p.mods[name] = &mod

	return &mod, nil
}

// GetMatchingRelease fetches information for the newest release matching the given dependency.
func (p *ModPortal) GetMatchingRelease(dep *Dependency) (*PortalModRelease, error) {
	mod, err := p.GetModInfo(dep.Name)
	if err != nil {
		return nil, err
	}
	// Iterate backwards to get the newest release first
	for i := len(mod.Releases) - 1; i >= 0; i-- {
		release := &mod.Releases[i]
		if dep.Test(&release.Version) && release.compatibleWithBaseVersion(p.baseVersion) {
			return release, nil
		}
	}

	return nil, ErrNoCompatibleRelease
}

// DownloadMatchingRelease downloads the latest mod release matching the given dependency.
// Returns the filepath of the newly downloaded mod.
func (p *ModPortal) DownloadMatchingRelease(dep *Dependency) (string, error) {
	if p.playerData.Token == "" {
		return "", errors.New("token was not specified")
	}
	if p.playerData.Username == "" {
		return "", errors.New("username was not specified")
	}
	release, err := p.GetMatchingRelease(dep)
	if err != nil {
		return "", err
	}

	downloadUrl := fmt.Sprintf(
		"%s/%s?username=%s&token=%s",
		p.server,
		release.DownloadUrl,
		p.playerData.Username,
		p.playerData.Token,
	)
	outPath := path.Join(p.downloadPath, release.FileName)

	fmt.Printf("downloading %s\n", release.FileName) // TODO: This doesn't belong here
	res, err := grab.Get(outPath, downloadUrl)
	if err != nil {
		return "", err
	}
	return res.Filename, nil
}

// DownloadLatestRelease downloads the latest release compatible with the current base version.
func (p *ModPortal) DownloadLatestRelease(name string) (string, error) {
	return p.DownloadMatchingRelease(&Dependency{
		Name: name,
		Kind: DependencyRequired,
		Req:  VersionAny,
	})
}

func (p *ModPortal) DownloadRelease(name string, version *Version) (string, error) {
	if version == nil {
		return p.DownloadLatestRelease(name)
	}
	return p.DownloadMatchingRelease(&Dependency{
		Name:    name,
		Version: version,
		Kind:    DependencyRequired,
		Req:     VersionEq,
	})
}

// UploadMod uploads the given file to the mod portal.
func (p *ModPortal) UploadMod(filepath string) error {
	// Init upload
	initUploadBody := &bytes.Buffer{}
	w := multipart.NewWriter(initUploadBody)
	ident := NewModIdent(path.Base(filepath))
	w.WriteField("mod", ident.Name)
	w.Close()
	url, err := url.JoinPath(p.server, "api/v2/mods/releases/init_upload")
	if err != nil {
		return err
	}
	req, err := http.NewRequest(http.MethodPost, url, initUploadBody)
	if err != nil {
		return err
	}
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", p.apiKey))
	req.Header.Set("Content-Type", w.FormDataContentType())
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	var decoded ModInitUploadRes
	err = json.NewDecoder(res.Body).Decode(&decoded)
	if err != nil {
		return err
	}
	if res.StatusCode != http.StatusOK {
		return errors.New(*decoded.Message)
	}
	defer res.Body.Close()

	// Open file
	file, err := os.Open(filepath)
	if err != nil {
		return err
	}
	defer file.Close()

	fmt.Printf("uploading %s\n", filepath) // TODO: Relocate this

	// Upload file
	uploadBody := &bytes.Buffer{}
	w = multipart.NewWriter(uploadBody)
	part, err := w.CreateFormFile("file", path.Base(file.Name()))
	io.Copy(part, file)
	w.Close()

	r, err := http.NewRequest("POST", *decoded.UploadUrl, uploadBody)
	if err != nil {
		return err
	}
	r.Header.Add("Content-Type", w.FormDataContentType())
	http.DefaultClient.Do(r)

	return nil
}

type ModInitUploadRes struct {
	UploadUrl *string `json:"upload_url"`
	Message   *string // When an error occurs
}

type PortalModInfo struct {
	Name     string
	Releases []PortalModRelease
	Title    string
}

type PortalModRelease struct {
	DownloadUrl string   `json:"download_url"`
	FileName    string   `json:"file_name"`
	InfoJson    infoJson `json:"info_json"`
	Version     Version  `json:"version"`
}

func (r *PortalModRelease) compatibleWithBaseVersion(baseVersion *Version) bool {
	for _, dep := range r.InfoJson.Dependencies {
		if dep.Name == "base" {
			if dep.Version == nil {
				return true
			}
			// Ensure that the Factorio versions match up as well
			return baseVersion[0] == dep.Version[0] && baseVersion[1] == dep.Version[1] && dep.Test(baseVersion)
		}
	}
	return true
}
