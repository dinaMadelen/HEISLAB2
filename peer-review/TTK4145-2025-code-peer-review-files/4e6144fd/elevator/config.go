package elevator

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

// Config-struktur for å lagre verdier fra config-filen
type Config struct {
	ClearRequestVariant ClearRequestVariant
	DoorOpenDurationS   float64
	InputPollRateMs     int
}

// LoadConfig leser `elevator.con` og setter verdier i en `Config`-struct
func LoadConfig(filename string) Config {
	config := Config{
		ClearRequestVariant: CV_All, // Standardverdi
		DoorOpenDurationS:   3.0,    // Standardverdi
		InputPollRateMs:     25,     // Standardverdi
	}

	file, err := os.Open(filename)
	if err != nil {
		fmt.Printf("Unable to open config file %s\n", filename)
		return config // Returnerer standardverdier hvis filen ikke finnes
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()

		// Kun linjer som starter med "--" skal behandles
		if !strings.HasPrefix(line, "--") {
			continue
		}

		// Del opp linjen i nøkkel og verdi
		fields := strings.Fields(line)
		if len(fields) < 2 {
			continue
		}

		key := strings.TrimPrefix(fields[0], "--")
		value := fields[1]

		// Match mot kjente konfigurasjonsparametere
		switch strings.ToLower(key) {
		case "clearrequestvariant":
			if strings.EqualFold(value, "CV_InDirn") {
				config.ClearRequestVariant = CV_InDirn
			} else {
				config.ClearRequestVariant = CV_All
			}

		case "dooropenduration_s":
			if v, err := strconv.ParseFloat(value, 64); err == nil {
				config.DoorOpenDurationS = v
			}

		case "inputpollrate_ms":
			if v, err := strconv.Atoi(value); err == nil {
				config.InputPollRateMs = v
			}
		}
	}

	return config
}
