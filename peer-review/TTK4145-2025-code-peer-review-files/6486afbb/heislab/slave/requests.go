package slave

import (
	"fmt"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/driver-go/elevio"
)

/*
Returns true if there are requests above the elevator's current floor

Else returns false
*/
func requests_above(elevator Elevator) bool {
	for f := elevator.Floor + 1; f < config.N_FLOORS; f++ {
		for btn := 0; btn < config.N_BUTTONS; btn++ {
			if elevator.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

/*
Returns true if there are requests below elevator's current floor

Else returns false
*/
func requests_below(elevator Elevator) bool {
	for f := 0; f < elevator.Floor; f++ {
		for btn := 0; btn < config.N_BUTTONS; btn++ {
			if elevator.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

/*
Returns true if there are requests at the elevator's current floor

Else returns false
*/
func requests_here(elevator Elevator) bool {
	for btn := 0; btn < config.N_BUTTONS; btn++ {
		if elevator.Requests[elevator.Floor][btn] {
			return true
		}
	}
	return false
}

/*
	Chooses the direction and behaviour of the elevator based on the requests and current state

Input: Original elevator object

Returns: Direction and behaviour the elevator should have
*/
func requests_chooseDirection(elevator Elevator) (ElevatorDirection, ElevatorBehaviour) {
	switch elevator.Direction {
	case D_Up:
		if requests_above(elevator) {
			return D_Up, EB_Moving
		} else if requests_here(elevator) {
			return D_Down, EB_DoorOpen
		} else if requests_below(elevator) {
			return D_Down, EB_Moving
		} else {
			return D_Stop, EB_Idle
		}
	case D_Down:
		if requests_below(elevator) {
			return D_Down, EB_Moving
		} else if requests_here(elevator) {
			return D_Up, EB_DoorOpen
		} else if requests_above(elevator) {
			return D_Up, EB_Moving
		} else {
			return D_Stop, EB_Idle
		}
	case D_Stop:
		if requests_here(elevator) {
			return D_Stop, EB_DoorOpen
		} else if requests_above(elevator) {
			return D_Up, EB_Moving
		} else if requests_below(elevator) {
			return D_Down, EB_Moving
		} else {
			return D_Stop, EB_Idle
		}
	}
	return D_Stop, EB_Idle
}

/*
Returns true if the elevator should stop at the current floor

Else returns false
*/
func requests_shouldStop(elevator Elevator) bool {
	switch elevator.Direction {
	case D_Down:
		if elevator.Requests[elevator.Floor][elevio.BT_HallDown] {
			return true
		}
		if elevator.Requests[elevator.Floor][elevio.BT_Cab] {
			return true
		}
		if !requests_below(elevator) {
			return true
		}
		return false
	case D_Up:
		if elevator.Requests[elevator.Floor][elevio.BT_HallUp] {
			return true
		}
		if elevator.Requests[elevator.Floor][elevio.BT_Cab] {
			return true
		}
		if !requests_above(elevator) {
			return true
		}
		return false
	default:
		return true
	}
}

/*
Clears requests depending on the direction of the elevator

Input: Original elevator object

Returns: New elevator object with cleared requests
*/
func requests_clearAtCurrentFloor(elevator Elevator) Elevator {
	elevator.Requests[elevator.Floor][elevio.BT_Cab] = false
	switch elevator.Direction {
	case D_Up:
		if !requests_above(elevator) && !elevator.Requests[elevator.Floor][elevio.BT_HallUp] {
			elevator.Requests[elevator.Floor][elevio.BT_HallDown] = false
		}
		elevator.Requests[elevator.Floor][elevio.BT_HallUp] = false
	case D_Down:
		if !requests_below(elevator) && !elevator.Requests[elevator.Floor][elevio.BT_HallDown] {
			elevator.Requests[elevator.Floor][elevio.BT_HallUp] = false
		}
		elevator.Requests[elevator.Floor][elevio.BT_HallDown] = false
	default:
		elevator.Requests[elevator.Floor][elevio.BT_HallUp] = false
		elevator.Requests[elevator.Floor][elevio.BT_HallDown] = false
	}
	fmt.Println("Cleared at current floor:", elevator.Requests)
	return elevator
}
