package requests

import (
	. "elevator/elevio"
)

type DirnBehaviourPair struct {
	Dirn      MotorDirection
	Behaviour ElevatorBehaviour
}

func RequestsAbove(e Elevator) bool {
	for f := e.Floor + 1; f < N_FLOORS; f++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func RequestsBelow(e Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func RequestsHere(e Elevator) bool {
	for btn := 0; btn < N_BUTTONS; btn++ {
		if e.Requests[e.Floor][btn] {
			return true
		}
	}
	return false
}

func RequestsChooseDirection(e Elevator) DirnBehaviourPair {
	switch e.Dirn {
	case MD_Up:
		if RequestsAbove(e) {
			return DirnBehaviourPair{Dirn: MD_Up, Behaviour: EB_Moving}
		}
		if RequestsHere(e) {
			return DirnBehaviourPair{Dirn: MD_Down, Behaviour: EB_DoorOpen}
		}
		if RequestsBelow(e) {
			return DirnBehaviourPair{Dirn: MD_Down, Behaviour: EB_Moving}
		}
		return DirnBehaviourPair{Dirn: MD_Stop, Behaviour: EB_Idle}
	case MD_Down:
		if RequestsAbove(e) {
			return DirnBehaviourPair{Dirn: MD_Up, Behaviour: EB_Moving}
		}
		if RequestsHere(e) {
			return DirnBehaviourPair{Dirn: MD_Up, Behaviour: EB_DoorOpen} // Er denne riktig?
		}
		if RequestsBelow(e) {
			return DirnBehaviourPair{Dirn: MD_Down, Behaviour: EB_Moving}
		}
		return DirnBehaviourPair{Dirn: MD_Stop, Behaviour: EB_Idle}
	case MD_Stop: // there should only be one request in the Stop case. Checking up or down first is arbitrary.
		if RequestsAbove(e) {
			return DirnBehaviourPair{Dirn: MD_Up, Behaviour: EB_Moving}
		}
		if RequestsHere(e) {
			return DirnBehaviourPair{Dirn: MD_Stop, Behaviour: EB_DoorOpen}
		}
		if RequestsBelow(e) {
			return DirnBehaviourPair{Dirn: MD_Down, Behaviour: EB_Moving}
		}
		return DirnBehaviourPair{Dirn: MD_Stop, Behaviour: EB_Idle}
	default:
		return DirnBehaviourPair{Dirn: MD_Stop, Behaviour: EB_Idle}
	}
}

func RequestsShouldStop(e Elevator) bool {
	switch e.Dirn {
	case MD_Down:
		return e.Requests[e.Floor][BT_HallDown] || e.Requests[e.Floor][BT_Cab] || !RequestsBelow(e)
	case MD_Up:
		return e.Requests[e.Floor][BT_HallUp] || e.Requests[e.Floor][BT_Cab] || !RequestsAbove(e)
	//case MD_Stop:
	default:
		return true
	}
}

func RequestsShouldClearImmediately(e Elevator, btn_floor int, btn_type ButtonType) bool {
	switch e.Config {
	case CV_All:
		return e.Floor == btn_floor
	case CV_InDirn:
		return e.Floor == btn_floor && ((e.Dirn == MD_Up && btn_type == BT_HallUp) || (e.Dirn == MD_Down && btn_type == BT_HallDown) || e.Dirn == MD_Stop || btn_type == BT_Cab)
	default:
		return false
	}
}

func RequestsClearAtCurrentFloor(e Elevator) Elevator {

	switch e.Config {
	case CV_All:
		for btn := 0; btn < N_BUTTONS; btn++ { //kan hende btn må være Button
			e.Requests[e.Floor][btn] = false
		}

	case CV_InDirn:
		e.Requests[e.Floor][BT_Cab] = false
		switch e.Dirn {
		case MD_Up:
			if !RequestsAbove(e) && !e.Requests[e.Floor][BT_HallUp] {
				e.Requests[e.Floor][BT_HallDown] = false
			}
			e.Requests[e.Floor][BT_HallUp] = false

		case MD_Down:
			if !RequestsBelow(e) && !e.Requests[e.Floor][BT_HallDown] {
				e.Requests[e.Floor][BT_HallUp] = false
			}
			e.Requests[e.Floor][BT_HallDown] = false

		case MD_Stop:
		default:
			e.Requests[e.Floor][BT_HallUp] = false
			e.Requests[e.Floor][BT_HallDown] = false

		}

	default:

	}

	return e
}
