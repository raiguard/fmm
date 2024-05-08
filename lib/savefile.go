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
	}
	if err != nil {
		return nil, err
	}
	defer rawReader.Close()

	datReader := newDatReader(rawReader)

	datReader.ReadUnoptimizedVersion()   // mapVersion
	datReader.ReadUint8()                // branchVersion
	datReader.ReadString()               // campaignName
	datReader.ReadString()               // levelName
	datReader.ReadString()               // modName
	datReader.ReadUint8()                // difficulty
	datReader.ReadBool()                 // finished
	datReader.ReadBool()                 // playerWon
	datReader.ReadString()               // nextLevel
	datReader.ReadBool()                 // canContinue
	datReader.ReadBool()                 // finishedButContinuing
	datReader.ReadBool()                 // savingReplay
	datReader.ReadBool()                 // allowNonAdminDebugOptions
	datReader.ReadOptimizedVersion(true) // scenarioVersion
	datReader.ReadUint8()                // scenarioBranchVersion
	datReader.ReadUint8()                // allowedCommands

	numMods := datReader.ReadUint16Optimized()
	mods := make([]ModIdent, numMods)
	for i := uint16(0); i < numMods; i += 1 {
		mods[i] = datReader.ReadModWithCRC()
	}

	datReader.ReadUint32() // startupModSettingsCrc

	pt, err := readPropertyTree(datReader)
	if err != nil {
		return nil, err
	}
	fmt.Printf("%+v\n", pt)

	return mods, nil
}
