package elevator

import (
	"Driver-go/elevio"
	"fmt"
)

// Direction represents the movement direction of the elevator.
type Direction int

const (
	Up   Direction = 0
	Down Direction = 1
)

// Converts Direction to elevio.MotorDirection
func (d Direction) ToMotorDirection() elevio.MotorDirection {
	switch d {
	case Up:
		return elevio.MD_Up
	case Down:
		return elevio.MD_Down
	default:
		fmt.Println("Warning: Invalid direction in ToMotorDirection(), returning MD_Stop")
		return elevio.MD_Stop
	}
}

// Converts Direction to elevio.ButtonType (used for button events)
func (d Direction) ToButtonType() elevio.ButtonType {
	switch d {
	case Up:
		return elevio.BT_HallUp
	case Down:
		return elevio.BT_HallDown
	default:
		fmt.Println("Warning: Invalid direction in ToButtonType(), returning invalid value")
		return -1 // Invalid ButtonType
	}
}

// Returns the opposite direction
func (d Direction) FlipDirection() Direction {
	if d != Up && d != Down {
		fmt.Println("Warning: Invalid direction in Opposite(), returning Up as default")
		return Up
	}
	return Direction(1 - d)
}

// Converts Direction to a readable string
func (d Direction) ToString() string {
	switch d {
	case Up:
		return "up"
	case Down:
		return "down"
	default:
		fmt.Println("Warning: Invalid direction in ToString(), returning 'unknown'")
		return "unknown"
	}
}
