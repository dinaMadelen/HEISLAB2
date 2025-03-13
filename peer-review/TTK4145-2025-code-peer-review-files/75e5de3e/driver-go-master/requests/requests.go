package requests

import (
	. "Driver-go/elevator"
)

type DirnBehaviourPair struct {
	Direction Dirn
	Behaviour ElevatorBehaviour
}


//Funksjon som sjekker om det er bestillinger i etasjer over
func RequestsAbove(e Elevator) bool {
	for f := e.Floor + 1; f < NFloors; f++ {
		for btn := 0; btn < NBtns; btn++ {
			if e.Requests[f][btn] == 1 {
				return true
			}
		}
	}
	return false
}


//Funksjon som sjekker om det er bestillinger i etasjer under
func RequestsBelow(e Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < NBtns; btn++ {
			if e.Requests[f][btn] == 1 {
				return true
			}
		}
	}
	return false
}


//Funksjon som sjekker om det er bestillinger i denne etasjen 
func RequestsHere(e Elevator) bool {
	for btn := 0; btn < NBtns; btn++ {
		if e.Requests[e.Floor][btn] == 1 {
			return true
		}
	}
	return false
}


//Funksjon som bestemmer oppførsel til heisen basert på bevegelsesretning og bestillinger
func RequestsChooseDirection(e Elevator) DirnBehaviourPair {
	switch e.Dirn {
	case Up:
		if RequestsAbove(e) {
			return DirnBehaviourPair{Direction: Up, Behaviour: Moving}
		} else if RequestsHere(e) {
			return DirnBehaviourPair{Direction: Down, Behaviour: DoorOpen}
		} else if RequestsBelow(e) {
			return DirnBehaviourPair{Direction: Down, Behaviour: Moving}
		}
		return DirnBehaviourPair{Direction: Stop, Behaviour: Idle}

	case Down:
		if RequestsBelow(e) {
			return DirnBehaviourPair{Direction: Down, Behaviour: Moving}
		} else if RequestsHere(e) {
			return DirnBehaviourPair{Direction: Up, Behaviour: DoorOpen}
		} else if RequestsAbove(e) {
			return DirnBehaviourPair{Direction: Up, Behaviour: Moving}
		}
		return DirnBehaviourPair{Direction: Stop, Behaviour: Idle}

	case Stop:
		if RequestsHere(e) {
			return DirnBehaviourPair{Direction: Stop, Behaviour: DoorOpen}
		} else if RequestsAbove(e) {
			return DirnBehaviourPair{Direction: Up, Behaviour: Moving}
		} else if RequestsBelow(e) {
			return DirnBehaviourPair{Direction: Down, Behaviour: Moving}
		}
		return DirnBehaviourPair{Direction: Stop, Behaviour: Idle}

	default:
		return DirnBehaviourPair{Direction: Stop, Behaviour: Idle}
	}
}


//Funksjon som sjekker om heisen skal stoppe i nåværende etasje
func RequestsShouldStop(e Elevator) bool {
	switch e.Dirn {
	case Down:
		if e.Requests[e.Floor][HallDown] == 1 ||
			e.Requests[e.Floor][Cab] == 1 ||
			!RequestsBelow(e) {
			return true
		} else {
			return false
		}

	case Up:
		if e.Requests[e.Floor][HallUp] == 1 ||
			e.Requests[e.Floor][Cab] == 1 ||
			!RequestsAbove(e) {
			return true
		} else {
			return false
		}
	case Stop:
		fallthrough
	default:
		return true
	}
}


//Funksjon som sletter en ny bestilling dersom bestillingen er fra nåværende etasje 
func RequestsShouldClearImmediately(e Elevator, btnFloor int) bool {
	return e.Floor == btnFloor
}


//Funksjon for å slette bestillinger som er utført på nåværende etasje
func RequestsClearAtCurrentFloor(e Elevator) Elevator {
	//Sletter cabcalls fra nåværende etasje
	e.Requests[e.Floor][Cab] = 0
	//Switch case for å bestemme hvilke hallcalls som skal slettes
	switch e.Dirn {
	case Up:
		if !RequestsAbove(e) && e.Requests[e.Floor][HallUp] == 0 {
			e.Requests[e.Floor][HallDown] = 0
		}
		e.Requests[e.Floor][HallUp] = 0

	case Down:
		if !RequestsBelow(e) && e.Requests[e.Floor][HallDown] == 0 {
			e.Requests[e.Floor][HallUp] = 0
		}
		e.Requests[e.Floor][HallDown] = 0

	case Stop:
		fallthrough
	default:
		e.Requests[e.Floor][HallUp] = 0
		e.Requests[e.Floor][HallDown] = 0
	}

	return e
}
