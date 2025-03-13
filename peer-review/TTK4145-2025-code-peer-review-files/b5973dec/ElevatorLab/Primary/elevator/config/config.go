package config

// --------------- SYSTEM WIDE CONFIGURATIONS --------------- //

const (
	NumFloors    int     = 4
	NumButtons   int     = 3
	DoorOpenTime float64 = 3.0
)

type ClearRequestVariant int

const (
	CV_All ClearRequestVariant = iota
	CV_InDirn
)

type ElevatorBehaviour int

const (
	EB_Idle = iota
	EB_DoorOpen
	EB_Moving
)

type Config struct {
	ClearRequestVariant ClearRequestVariant
	DoorOpenDuration_s  float64
}
