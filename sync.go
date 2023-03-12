package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func syncWithLog(filepath string) error {
	file, err := os.Open(filepath)
	if err != nil {
		return err
	}
	defer file.Close()

	fileScanner := bufio.NewScanner(file)
	fileScanner.Split(bufio.ScanLines)

	var modNames []string
	inChecksums := false
	for fileScanner.Scan() {
		line := fileScanner.Text()
		if !strings.Contains(line, "Checksum of") {
			if inChecksums {
				break
			} else {
				continue
			}
		}
		inChecksums = true
		modName, _ := strings.CutSuffix(strings.Split(strings.TrimSpace(line), " ")[3], ":")
		if modName == "base" {
			continue
		}
		modNames = append(modNames, modName)
	}

	for _, modname := range modNames {
		fmt.Println(modname)
	}

	download(modNames)
	enable(modNames)

	return nil
}
