package requests

import (
	"Driver-go/elevio"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/elevator"
)

type DirBehaviorPair struct {
	Dirn      int
	Behaviour int
}

func above(e elevator.Elevator) bool {
	for f := e.Floor + 1; f < elevator.NumFloors; f++ {
		for btn := 0; btn < elevator.NumButtons; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func below(e elevator.Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < elevator.NumButtons; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func here(e elevator.Elevator) bool {
	for btn := 0; btn < elevator.NumButtons; btn++ {
		if e.Requests[e.Floor][btn] {
			return true
		}
	}
	return false
}

func ChooseDirection(e elevator.Elevator) DirBehaviorPair {
	switch e.Dirn {
	case elevator.D_Up:
		if above(e) {
			return DirBehaviorPair{Dirn: elevator.D_Up, Behaviour: elevator.EB_Moving}
		}
		if here(e) {
			return DirBehaviorPair{Dirn: elevator.D_Down, Behaviour: elevator.EB_DoorOpen}
		}
		if below(e) {
			return DirBehaviorPair{Dirn: elevator.D_Down, Behaviour: elevator.EB_Moving}
		}
		return DirBehaviorPair{Dirn: elevator.D_Stop, Behaviour: elevator.EB_Idle}
	case elevator.D_Down:
		if below(e) {
			return DirBehaviorPair{Dirn: elevator.D_Down, Behaviour: elevator.EB_Moving}
		}
		if here(e) {
			return DirBehaviorPair{Dirn: elevator.D_Up, Behaviour: elevator.EB_DoorOpen}
		}
		if above(e) {
			return DirBehaviorPair{Dirn: elevator.D_Up, Behaviour: elevator.EB_Moving}
		}
		return DirBehaviorPair{Dirn: elevator.D_Stop, Behaviour: elevator.EB_Idle}
	case elevator.D_Stop:
		if here(e) {
			return DirBehaviorPair{Dirn: elevator.D_Stop, Behaviour: elevator.EB_DoorOpen}
		}
		if above(e) {
			return DirBehaviorPair{Dirn: elevator.D_Up, Behaviour: elevator.EB_Moving}
		}
		if below(e) {
			return DirBehaviorPair{Dirn: elevator.D_Down, Behaviour: elevator.EB_Moving}
		}
		return DirBehaviorPair{Dirn: elevator.D_Stop, Behaviour: elevator.EB_Idle}
	default:
		return DirBehaviorPair{Dirn: elevator.D_Stop, Behaviour: elevator.EB_Idle}
	}
}

func ShouldStop(e elevator.Elevator) bool {
	switch e.Dirn {
	case elevator.D_Down:
		return (e.Requests[e.Floor][elevio.BT_HallDown] ||
			e.Requests[e.Floor][elevio.BT_Cab] ||
			!below(e))
	case elevator.D_Up:
		return (e.Requests[e.Floor][elevio.BT_HallUp] ||
			e.Requests[e.Floor][elevio.BT_Cab] ||
			!above(e))
	case elevator.D_Stop:
		break
	default:
		return true
	}
	return false
}

func ShouldClearImmediately(e elevator.Elevator, btn_floor int, btn_type elevio.ButtonType) bool {
	switch e.Config.ClearRequestVariant {
	case "CV_All":
		return e.Floor == btn_floor
	case "CV_InDirn":
		return e.Floor == btn_floor && ((e.Dirn == elevator.D_Up && btn_type == elevio.BT_HallUp) || (e.Dirn == elevator.D_Down && btn_type == elevio.BT_HallDown) || e.Dirn == elevator.D_Stop || btn_type == elevio.BT_Cab)
	default:
		return false
	}
}

func ClearAtCurrentFloor(e elevator.Elevator) elevator.Elevator {

	switch e.Config.ClearRequestVariant {
	case "CV_All":
		for btn := 0; btn < elevator.NumButtons; btn++ { //Not entirely accurate to the source, btn is of type ButtonType, not int
			e.Requests[e.Floor][btn] = false
		}
		break
	case "CV_InDirn":
		e.Requests[e.Floor][elevio.BT_Cab] = false
		switch e.Dirn {
		case elevator.D_Up:
			if !above(e) && !e.Requests[e.Floor][elevio.BT_HallUp] {
				e.Requests[e.Floor][elevio.BT_HallDown] = false
			}
			e.Requests[e.Floor][elevio.BT_HallUp] = false
			break
		case elevator.D_Down:
			if !below(e) && !e.Requests[e.Floor][elevio.BT_HallDown] {
				e.Requests[e.Floor][elevio.BT_HallUp] = false
			}
			e.Requests[e.Floor][elevio.BT_HallDown] = false
			break
		case elevator.D_Stop:
			e.Requests[e.Floor][elevio.BT_HallUp] = false
			e.Requests[e.Floor][elevio.BT_HallDown] = false
		default:
			e.Requests[e.Floor][elevio.BT_HallUp] = false
			e.Requests[e.Floor][elevio.BT_HallDown] = false
			break
		}
		break
	default:
		break
	}

	return e
}
