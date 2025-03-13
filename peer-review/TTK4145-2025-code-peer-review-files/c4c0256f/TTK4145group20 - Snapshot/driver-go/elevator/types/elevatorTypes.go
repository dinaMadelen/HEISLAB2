package types

import "sync"

const (
	N_FLOORS             = 4 // Total number of floors in the elevator
	N_BUTTONS            = 3 // Number of button types (HallUp, HallDown, Cab)
	N_HALL_BUTTONS       = 2 // Up, down
	DOOR_TIMEOUT_SEC     = 3
	MOBILITY_TIMEOUT_SEC = 4
)

type ElevBehaviour int

const (
	EB_Idle     ElevBehaviour = iota // Elevator is idle (not moving and not opening doors)
	EB_DoorOpen                      // Doors are open
	EB_Moving                        // Elevator is moving between floors
)

type ElevDirection int

const (
	ED_Down ElevDirection = iota // Moving down
	ED_Stop                      // Stopped (idle state)
	ED_Up                        // Moving up
)

type ClearRequestVariant int

const (
	CV_All    ClearRequestVariant = iota // Everyone enters the elevator, even if going in the "wrong" direction
	CV_InDirn                            // Only passengers traveling in the current direction enter
)

type Config struct {
	DoorOpenDuration_s  float64             // Duration (in seconds) that the door remains open after a request
	ClearRequestVariant ClearRequestVariant // Defines how requests are cleared
}

type Elevator struct {
	Floor     int
	Direction ElevDirection
	Requests  [N_FLOORS][N_BUTTONS]bool
	Behaviour ElevBehaviour
	Config    Config
}

type ElevatorSharedInfo struct {
	Mutex     sync.RWMutex
	Available bool
	Behaviour ElevBehaviour
	Direction ElevDirection
	Floor     int
}

func InitElevator() Elevator {
	return Elevator{
		Floor:     -1,      // Uninitialized floor
		Direction: ED_Stop, // Initial direction is stopped
		Behaviour: EB_Idle, // Initial behavior is idle

		Config: Config{
			ClearRequestVariant: CV_InDirn, // Default clearing behavior: all requests are handled
			DoorOpenDuration_s:  DOOR_TIMEOUT_SEC,       // Default duration for door open: 1.5 seconds
		},
	}

}
