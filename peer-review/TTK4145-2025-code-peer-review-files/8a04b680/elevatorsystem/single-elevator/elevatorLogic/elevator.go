package elevatorLogic

import (
	"elevatorsystem/single-elevator/Driver-go/elevio"
	"elevatorsystem/constants"
)

type ElevatorBehaviour int

const (
	EB_Idle ElevatorBehaviour = iota
	EB_DoorOpen
	EB_Moving
)

// Datatype determining when the elevator should clear requests
type ClearOrderVariant int

const (
	// Assume everyone waiting for the elevator gets on the elevator, even if
	// they will be traveling in the "wrong" direction for a while
	CV_All ClearOrderVariant = iota

	// Assume that only those that want to travel in the current direction
	// enter the elevator, and keep waiting outside otherwise
	CV_InDirection
)

type config struct {
	ClearOrderVariant  ClearOrderVariant
	DoorOpenDuration_s float64
}

type Elevator struct {
	Orders         [constants.NUM_FLOORS][constants.NUM_BUTTONS]bool
	LastKnownFloor int
	Direction      elevio.MotorDirection
	Behaviour      ElevatorBehaviour
	ElevatorID     string

	Config config
}

func ElevatorUninitialized() Elevator {
	elevator := Elevator{
		LastKnownFloor: -1,
		Direction:      elevio.MD_Stop,
		Behaviour:      EB_Idle,
		ElevatorID:     "",
		Config: config{
			ClearOrderVariant:  CV_InDirection,
			DoorOpenDuration_s: 3.0,
		},
	}
	return elevator
}
