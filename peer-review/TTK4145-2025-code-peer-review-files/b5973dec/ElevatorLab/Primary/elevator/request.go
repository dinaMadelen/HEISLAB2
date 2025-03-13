package elevator

import (
	"Primary/elevator/config"
	"Primary/elevator/elevio"
)

// --------------- REQUEST FUNCTIONS --------------- //

// --- LOCAL FUNCTIONS --- //

func requests_above(e Elevator) bool {
	for f := e.Floor + 1; f < config.NumFloors; f++ {
		for btn := 0; btn < config.NumButtons; btn++ {
			if e.Request[f][btn] == true {
				return true
			}
		}
	}
	return false
}

func requests_below(e Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < config.NumButtons; btn++ {
			if e.Request[f][btn] == true {
				return true
			}
		}
	}
	return false
}

func requests_here(e Elevator) bool {
	for btn := 0; btn < config.NumButtons; btn++ {
		if e.Request[e.Floor][btn] == true {
			return true
		}
	}
	return false
}

func request_chooseDirection(e Elevator) DirnBehaviourPair {
	switch e.Dirn {
	case elevio.MD_Up:
		if requests_above(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Up, behaviour: config.EB_Moving}
		} else if requests_here(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Down, behaviour: config.EB_DoorOpen}
		} else if requests_below(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Down, behaviour: config.EB_Moving}
		} else {
			return DirnBehaviourPair{dirn: elevio.MD_Stop, behaviour: config.EB_Idle}
		}
	case elevio.MD_Down:
		if requests_below(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Down, behaviour: config.EB_Moving}
		} else if requests_here(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Up, behaviour: config.EB_DoorOpen}
		} else if requests_above(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Up, behaviour: config.EB_Moving}
		} else {
			return DirnBehaviourPair{dirn: elevio.MD_Stop, behaviour: config.EB_Idle}
		}
	case elevio.MD_Stop:
		if requests_here(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Stop, behaviour: config.EB_DoorOpen}
		} else if requests_above(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Up, behaviour: config.EB_Moving}
		} else if requests_below(e) {
			return DirnBehaviourPair{dirn: elevio.MD_Down, behaviour: config.EB_Moving}
		} else {
			return DirnBehaviourPair{dirn: elevio.MD_Stop, behaviour: config.EB_Idle}
		}
	default:
		return DirnBehaviourPair{dirn: elevio.MD_Stop, behaviour: config.EB_Idle}
	}

}

func requests_shouldStop(e Elevator) bool {
	switch e.Dirn {
	case elevio.MD_Down:
		if e.Request[e.Floor][elevio.BT_HallDown] == true || e.Request[e.Floor][elevio.BT_Cab] == true || !requests_below(e) {
			return true
		}

	case elevio.MD_Up:
		if e.Request[e.Floor][elevio.BT_HallUp] == true || e.Request[e.Floor][elevio.BT_Cab] == true || !requests_above(e) {
			return true
		}
	case elevio.MD_Stop:
		return true
	default:
		return true
	}
	return false
}

func requests_shouldClearImmediatly(e Elevator, btn_Floor int, btn_type elevio.ButtonType) bool {
	switch e.Config.ClearRequestVariant {
	case config.CV_All:
		return (e.Floor == btn_Floor)
	case config.CV_InDirn:
		return e.Floor == btn_Floor && ((e.Dirn == elevio.MD_Up && btn_type == elevio.BT_HallUp) || (e.Dirn == elevio.MD_Down && btn_type == elevio.BT_HallDown) || e.Dirn == elevio.MD_Stop || btn_type == elevio.BT_Cab)
	default:
		return false
	}

}

func requests_clearAtCurrentFloor(e Elevator) Elevator {

	switch e.Config.ClearRequestVariant {
	case config.CV_All:
		for btn := 0; btn < config.NumButtons; btn++ {
			e.Request[e.Floor][btn] = false
		}

	case config.CV_InDirn:

		e.Request[e.Floor][elevio.BT_Cab] = false

		switch e.Dirn {
		case elevio.MD_Up:

			if !requests_above(e) && !(e.Request[e.Floor][elevio.BT_HallUp] == true) {
				e.Request[e.Floor][elevio.BT_HallDown] = false
			}

			e.Request[e.Floor][elevio.BT_HallUp] = false

		case elevio.MD_Down:

			if !requests_below(e) && !(e.Request[e.Floor][elevio.BT_HallDown] == true) {
				e.Request[e.Floor][elevio.BT_HallUp] = false
			}

			e.Request[e.Floor][elevio.BT_HallDown] = false

		case elevio.MD_Stop:
			e.Request[e.Floor][elevio.BT_HallUp] = false
			e.Request[e.Floor][elevio.BT_HallDown] = false

		default:
			e.Request[e.Floor][elevio.BT_HallUp] = false
			e.Request[e.Floor][elevio.BT_HallDown] = false

		}

	default:
		break
	}
	return e
}

// --- GLOBAL FUNCTIONS --- //

func Request_timeToServe(ele_old Elevator, request elevio.ButtonEvent) int {

	travelTime_s := 2
	ele := ele_old
	ele.Request[request.Floor][request.Button] = true

	duration := 0

	switch ele.Behaviour {
	case config.EB_Idle:
		ele.Dirn = request_chooseDirection(ele).dirn
		if ele.Dirn == elevio.MD_Stop {
			return duration
		}
		break
	case config.EB_Moving:
		duration += int(ele.Config.DoorOpenDuration_s) / 2
		ele.Floor += int(ele.Dirn)
		break
	case config.EB_DoorOpen:
		duration -= int(ele.Config.DoorOpenDuration_s) / 2
	}

	for {
		if requests_shouldStop(ele) {

			ele = requests_clearAtCurrentFloor(ele)

			if !ele.Request[request.Floor][request.Button] {
				return duration
			}

			duration += int(ele.Config.DoorOpenDuration_s)
			ele.Dirn = request_chooseDirection(ele).dirn
		}
		ele.Floor += int(ele.Dirn)
		duration += travelTime_s
	}
}
