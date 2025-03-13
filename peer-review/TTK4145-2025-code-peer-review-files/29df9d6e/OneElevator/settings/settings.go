package settings

import "time"

const (
	Buffer            = 1024
	HeartbeatInterval = 15 * time.Millisecond
	Timeout           = 500 * time.Millisecond
	DoorOpenDuration  = 3 * time.Second

	NumFloors      = 4
	NumButtonTypes = 3
	NumElevators   = 3
)
