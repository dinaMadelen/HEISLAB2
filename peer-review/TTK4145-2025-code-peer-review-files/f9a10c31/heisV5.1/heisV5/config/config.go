package config

import "time"

var ElevatorID int

// Antall etasjer og heiser i systemet
const (
	NumFloors    = 4
	NumElevators = 3 // Antall heiser i nettverket
	NumButtons   = 3 //  Opp, ned og kabinknapp
)

// Nettverksinnstillinger
const (
	PeersPortNumber = 58735 //  Port for heis-til-heis kommunikasjon
	BcastPortNumber = 58750 // Port for brodcast
	BufferSize      = 1024  // Størrelse på nettverksbuffer
)

// Tidskonstanter
const (
	DisconnectTime   = 1 * time.Second       // Maks tid før en heis regnes som frakoblet
	DoorOpenDuration = 3 * time.Second       // Hvor lenge døren skal være åpen
	WatchdogTime     = 4 * time.Second       // Overvåkningstimer for feil
	HeartbeatTime    = 15 * time.Millisecond // Hvor ofte en heis sender "jeg lever"-melding
)
