package requests

import (
	"ElevatorProject/elevator"
	"ElevatorProject/elevio"
)

type DirnBehaviourPair struct {
	Dirn      elevio.MotorDirection
	Behaviour elevio.Behaviour
}

func requestsAbove(e elevator.ElevatorObject) bool {
	// Iterate through all floors above current floor
	for i := e.Floor + 1; i < elevio.NumFloors; i++ {
		// check button requests for all floors
		for j := 0; j < elevio.NumButtonTypes; j++ {
			if e.Requests[i][j] == 1 { // If there is an active request
				return true
			}
		}
	}
	return false
}

func requestsBelow(e elevator.ElevatorObject) bool {
	// Iterate through all floors below current floor
	for i := 0; i < e.Floor; i++ {
		// check button requests for all floors
		for j := 0; j < elevio.NumButtonTypes; j++ {
			if e.Requests[i][j] == 1 { // If there is an active request
				return true
			}
		}
	}
	return false
}

func requestsHere(e elevator.ElevatorObject) bool {
	// Checking all button request in current floor
	for j := 0; j < elevio.NumButtonTypes; j++ {
		if e.Requests[e.Floor][j] == 1 { // If there is an active request
			return true
		}
	}
	return false
}

func RequestsChooseDirection(e elevator.ElevatorObject) DirnBehaviourPair {
	switch e.Dirn {
	case elevio.MD_Up:
		if requestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else if requestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_DoorOpen}
		} else if requestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}
	case elevio.MD_Down:
		if requestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else if requestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_DoorOpen}
		} else if requestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}
	case elevio.MD_Stop:
		if requestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_DoorOpen}
		} else if requestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else if requestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}
	default:
		return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
	}
}

func RequestsShouldStop(e elevator.ElevatorObject) bool {
	switch e.Dirn {
	case elevio.MD_Down:
		return e.Requests[e.Floor][elevio.BT_HallDown] == 1 ||
			e.Requests[e.Floor][elevio.BT_Cab] == 1 ||
			!requestsBelow(e)
	case elevio.MD_Up:
		return e.Requests[e.Floor][elevio.BT_HallUp] == 1 ||
			e.Requests[e.Floor][elevio.BT_Cab] == 1 ||
			!requestsAbove(e)
	case elevio.MD_Stop:
		fallthrough // Go to default
	default:
		return true
	}
}

func RequestsShouldClearImmediately(e elevator.ElevatorObject, btnFloor int, btnType elevio.ButtonType) bool {
	switch e.Config.ClearRequestVariant {
	case elevator.CV_All:
		// Clear request if elevator is on this floor
		return e.Floor == btnFloor
	case elevator.CV_InDirn:
		// Clear request if elevator is on this floor and:
		return e.Floor == btnFloor &&
			((e.Dirn == elevio.MD_Up && btnType == elevio.BT_HallUp) ||
				(e.Dirn == elevio.MD_Down && btnType == elevio.BT_HallDown) ||
				e.Dirn == elevio.MD_Stop ||
				btnType == elevio.BT_Cab)
	default:
		// Do not clear request
		return false
	}
}

func RequestsClearAtCurrentFloor(e elevator.ElevatorObject) elevator.ElevatorObject {
	switch e.Config.ClearRequestVariant {
	case elevator.CV_All:
		// Clear all requests on current floor
		for btn := 0; btn < elevio.NumButtonTypes; btn++ {
			e.Requests[e.Floor][btn] = 0
		}

	case elevator.CV_InDirn:
		// Clear all requests in cab
		e.Requests[e.Floor][elevio.BT_Cab] = 0

		switch e.Dirn {
		case elevio.MD_Up:
			if !requestsAbove(e) && e.Requests[e.Floor][elevio.BT_Cab] == 0 {
				e.Requests[e.Floor][elevio.BT_Cab] = 0
			}
			e.Requests[e.Floor][elevio.BT_Cab] = 0

		case elevio.MD_Down:
			if !requestsBelow(e) && e.Requests[e.Floor][elevio.BT_Cab] == 0 {
				e.Requests[e.Floor][elevio.BT_Cab] = 0
			}
			e.Requests[e.Floor][elevio.BT_Cab] = 0

		case elevio.MD_Stop:
			fallthrough
		default:
			e.Requests[e.Floor][elevio.BT_HallUp] = 0
			e.Requests[e.Floor][elevio.BT_HallDown] = 0
		}
	}
	return e
}
