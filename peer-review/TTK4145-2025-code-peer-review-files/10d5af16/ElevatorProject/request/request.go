package request

import (
	"Driver-go/elevio"
	"elevator"
	"fmt"
)

type DirnBehaviourPair struct {
	Dirn      elevio.MotorDirection
	Behaviour elevator.ElevatorBehaviour
}


func Clear_all_requests(e elevator.Elevator) {
	fmt.Printf("Clearing all requests!\n")
	for f := 0; f < elevator.N_FLOORS; f++ {
		for b := 0; b < elevator.N_BUTTONS; b++ {
			e.RequestsToDo[f][b] = false
		}
	}
	for f := 0; f < elevator.N_FLOORS; f++ {
		for b := 0; b < elevator.N_BUTTONS; b++ {
			if e.RequestsToDo[f][b] {
				fmt.Printf("Order at floor %d not cleared", f)
			}
		}
	}
}


func Requests_above(e elevator.Elevator) bool {
	for f := e.Floor + 1; f < elevator.N_FLOORS; f++ {
		for btn := 0; btn < elevator.N_BUTTONS; btn++ {
			if e.RequestsToDo[f][btn] {
				return true
			}
		}
	}
	return false
}


func Requests_below(e elevator.Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < elevator.N_BUTTONS; btn++ {
			if e.RequestsToDo[f][btn] {
				return true
			}
		}
	}
	return false
}


func Requests_here(e elevator.Elevator) bool {
	for btn := 0; btn < elevator.N_BUTTONS; btn++ {
		if e.RequestsToDo[e.Floor][btn] {
			return true
		}
	}
	return false
}

// Ikke så stor fan av hva som skjer inne i hver case her
// Kanskje det løses bedre med enda en switch case?
func Requests_chooseDirection(e elevator.Elevator) DirnBehaviourPair {
	switch e.Dirn {
		// Leter først etter ordre i samme retning
	case elevio.MD_Up:
		if Requests_above(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else if Requests_here(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_DoorOpen}
		} else if Requests_below(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}

	case elevio.MD_Down:
		if Requests_below(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else if Requests_here(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_DoorOpen}
		} else if Requests_above(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}

	case elevio.MD_Stop:
		if Requests_here(e) {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_DoorOpen}
		} else if Requests_below(e) {
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
		} else if Requests_above(e) {
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
		}
	default:
		return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
	}
}

func Requests_shouldStop(e elevator.Elevator) bool {
	switch e.Dirn {
	case elevio.MD_Down:
		return e.RequestsToDo[e.Floor][elevator.B_HallDown] ||
			e.RequestsToDo[e.Floor][elevator.B_Cab] ||
			!Requests_below(e)
	case elevio.MD_Up:
		return e.RequestsToDo[e.Floor][elevator.B_HallUp] ||
			e.RequestsToDo[e.Floor][elevator.B_Cab] ||
			!Requests_above(e)
	default:
		return true
	}
}

func Requests_shouldClearImmediately(e elevator.Elevator, btn_floor int, btn_type elevio.ButtonType) bool {
	switch e.Config.ClearRequestVariant {
	case "CV_All":
		return e.Floor == btn_floor

	case "CV_InDirn":
		return e.Floor == btn_floor && ((e.Dirn == elevio.MD_Up && btn_type == elevator.B_HallUp) ||
			(e.Dirn == elevio.MD_Down && btn_type == elevator.B_HallDown) ||
			e.Dirn == elevio.MD_Stop ||
			btn_type == elevator.B_Cab)

	default:
		return false
	}
}

func Requests_clearAtCurrentFloor(e elevator.Elevator) elevator.Elevator {
	switch e.Config.ClearRequestVariant {
	case "CV_All":
		for btn := 0; btn < elevator.N_BUTTONS; btn++ {
			e.RequestsToDo[e.Floor][btn] = false
		}

	case "CV_InDirn":
		e.RequestsToDo[e.Floor][elevator.B_Cab] = false

		switch e.Dirn {
		case elevio.MD_Up:
			if !Requests_above(e) && !e.RequestsToDo[e.Floor][elevator.B_HallUp] {
				e.RequestsToDo[e.Floor][elevator.B_HallDown] = false
			}
			e.RequestsToDo[e.Floor][elevator.B_HallUp] = false

		case elevio.MD_Down:
			if !Requests_below(e) && !e.RequestsToDo[e.Floor][elevator.B_HallDown] {
				e.RequestsToDo[e.Floor][elevator.B_HallUp] = false
			}
			e.RequestsToDo[e.Floor][elevator.B_HallDown] = false

		default:
			e.RequestsToDo[e.Floor][elevator.B_HallUp] = false
			e.RequestsToDo[e.Floor][elevator.B_HallDown] = false
		}

	default:
		break
	}

	return e
}
