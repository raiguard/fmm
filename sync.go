package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func parseLogFile(filepath string) []ModIdent {
	var output []ModIdent
	file, err := os.Open(filepath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error reading %s: %s", filepath, err)
		return output
	}
	defer file.Close()

	fileScanner := bufio.NewScanner(file)
	fileScanner.Split(bufio.ScanLines)

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
		output = append(output, ModIdent{modName, nil})
	}

	return output
}
