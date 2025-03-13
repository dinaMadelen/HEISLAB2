package config

import (
	"encoding/json"
	"fmt"
	"os"
)

// Config defines the structure of the configuration
type Config struct {
	NumFloors  int `json:"NumFloors"`
	NumButtons int `json:"NumButtons"`
}

// Use to load Heis/config/elevator_params.json
func LoadConfig(filePath string) (*Config, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, fmt.Errorf("could not open config file: %v", err)
	}
	defer file.Close()

	config := &Config{}
	decoder := json.NewDecoder(file)
	if err := decoder.Decode(config); err != nil {
		return nil, fmt.Errorf("could not decode config JSON: %v", err)
	}

	return config, nil
}

const NumFloors = 4
const NumButtons = 3
const DoorOpenDuration = 3
const StateUpdatePeriodMs = 500
const ElevatorStuckToleranceSec = 5
const ReconnectTimerSec = 3
const LocalElevator = 0
