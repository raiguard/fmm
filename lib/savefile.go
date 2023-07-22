package fmm

import (
	"archive/zip"
	"compress/zlib"
	"errors"
	"io"
	"strings"
)

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
		return nil, errors.New("Invalid save file: could not locate level data")
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

	bytes, err := io.ReadAll(rawReader)
	if err != nil {
		return nil, err
	}

	datReader := newDatReader(bytes)

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
	// TODO: Startup mod settings PropertyTree

	return mods, nil
}
