package fmm

import (
	"archive/zip"
	"compress/zlib"
	"errors"
	"fmt"
	"strings"
)

// Returns a slice of mod names and versions extracted from the given save
// file.
func ParseSaveFile(filepath string) ([]ModIdent, error) {
	zipReader, err := zip.OpenReader(filepath)
	if err != nil {
		return nil, err
	}
	defer zipReader.Close()

	var dat *zip.File
	compressed := false
	for _, file := range zipReader.File {
		parts := strings.Split(file.Name, "/")
		name := parts[len(parts)-1]
		if name == "level.dat" || name == "level.dat0" {
			dat = file
			compressed = name == "level.dat0"
			break
		}
	}
	if dat == nil {
		return nil, errors.New("invalid save file: could not locate level data")
	}

	rawReader, err := dat.Open()
	if err != nil {
		return nil, err
	}
	if compressed {
		rawReader, err = zlib.NewReader(rawReader)
		if err != nil {
			return nil, err
		}
	}
	defer rawReader.Close()

	r := newDatReader(rawReader)

	r.ReadUnoptimizedVersion()   // mapVersion
	r.ReadUint8()                // branchVersion
	r.ReadString()               // campaignName
	r.ReadString()               // levelName
	r.ReadString()               // modName
	r.ReadUint8()                // difficulty
	r.ReadBool()                 // finished
	r.ReadBool()                 // playerWon
	r.ReadString()               // nextLevel
	r.ReadBool()                 // canContinue
	r.ReadBool()                 // finishedButContinuing
	r.ReadBool()                 // savingReplay
	r.ReadBool()                 // allowNonAdminDebugOptions
	r.ReadOptimizedVersion(true) // scenarioVersion
	r.ReadUint8()                // scenarioBranchVersion
	r.ReadUint8()                // allowedCommands

	numMods := r.ReadUint16Optimized()
	mods := make([]ModIdent, numMods)
	for i := uint16(0); i < numMods; i += 1 {
		mods[i] = r.ReadModWithCRC()
	}

	r.ReadUint32() // startupModSettingsCrc

	pt := r.ReadPropertyTree()
	fmt.Printf("%+v\n", pt)

	return mods, nil
}
