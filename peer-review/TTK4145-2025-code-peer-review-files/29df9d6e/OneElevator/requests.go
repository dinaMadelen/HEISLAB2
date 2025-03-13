package main

import (
	"OneElevator/elevio"
)

type DirnBehaviourPair struct {
	Dirn      elevio.MotorDirection
	Behaviour ElevatorBehaviour
}

func RequestsAbove(e Elevator) bool {
	// Iterer gjennom etasjene over nåværende etasje
	for i := e.Floor + 1; i < N_FLOORS; i++ {
		// Sjekk alle knappeforespørsler for hver etasje
		for j := 0; j < N_BUTTONS; j++ {
			if e.Requests[i][j] == 1 { // Hvis det finnes en aktiv forespørsel
				return true
			}
		}
	}
	return false
}

func RequestsBelow(e Elevator) bool {
	// Iterer gjennom etasjene under nåværende etasje
	for i := 0; i < e.Floor; i++ {
		// Sjekk alle knappeforespørsler for hver etasje
		for j := 0; j < N_BUTTONS; j++ {
			if e.Requests[i][j] == 1 { // Hvis det finnes en aktiv forespørsel
				return true
			}
		}
	}
	return false
}

// Sjekker requests på nåvæerende etasje
func RequestsHere(e Elevator) bool {
	// Sjekk alle knappeforespørsler for den nåværende etasjen
	for j := 0; j < N_BUTTONS; j++ {
		if e.Requests[e.Floor][j] == 1 { // Hvis det finnes en aktiv forespørsel
			return true
		}
	}
	return false
}

// avgjør ny retning og oppførsel til heisen
func ChooseDirection(e Elevator) DirnBehaviourPair {
	switch e.Dirn {
	case elevio.MD_Up:
		if RequestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, EB_Moving}
		
			} else if RequestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Stop, EB_DoorOpen}
		
			} else if RequestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, EB_Moving}
		
			} else {
			return DirnBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	case elevio.MD_Down:
		if RequestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, EB_Moving}
		
			} else if RequestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Stop, EB_DoorOpen}

			} else if RequestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, EB_Moving}
		
			} else {
			return DirnBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	case elevio.MD_Stop:
		if RequestsHere(e) {
			return DirnBehaviourPair{elevio.MD_Stop, EB_DoorOpen}
		} else if RequestsAbove(e) {
			return DirnBehaviourPair{elevio.MD_Up, EB_Moving}
		} else if RequestsBelow(e) {
			return DirnBehaviourPair{elevio.MD_Down, EB_Moving}
		} else {
			return DirnBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	default:
		return DirnBehaviourPair{elevio.MD_Stop, EB_Idle}
	}
}

// ShouldStop determines if the elevator should stop at the current floor
func ShouldStop(e Elevator) bool {
	switch e.Dirn {
	case elevio.MD_Down:
		return e.Requests[e.Floor][elevio.BT_HallDown] == 1 ||
			e.Requests[e.Floor][elevio.BT_Cab] == 1 ||
			!RequestsBelow(e)
	case elevio.MD_Up:
		return e.Requests[e.Floor][elevio.BT_HallUp] == 1 ||
			e.Requests[e.Floor][elevio.BT_Cab] == 1 ||
			!RequestsAbove(e)
	case elevio.MD_Stop:
		fallthrough // Gå til default
	default:
		return true
	}
}

func ShouldClearImmediately(e Elevator, btnFloor int, btnType elevio.ButtonType) bool {
	switch e.Config.ClearMode {
	case ClearAll:
		// Fjern forespørselen hvis heisen er i samme etasje
		return e.Floor == btnFloor
	case ClearDirectional:
		// Fjern forespørselen hvis heisen er i samme etasje og:
		return e.Floor == btnFloor &&
			((e.Dirn == elevio.MD_Up && btnType == elevio.BT_HallUp) ||
				(e.Dirn == elevio.MD_Down && btnType == elevio.BT_HallDown) ||
				e.Dirn == elevio.MD_Stop ||
				btnType == elevio.BT_Cab)
	default:
		// Ikke fjern forespørselen
		return false
	}
}

func ClearAtCurrentFloor(e Elevator) Elevator {
	switch e.Config.ClearMode {
	case ClearAll:
		// Fjern alle forespørsler i den nåværende etasjen
		for btn := 0; btn < N_BUTTONS; btn++ {
			e.Requests[e.Floor][btn] = 0
		}

	case ClearDirectional:
		// Fjern forespørsler fra innsiden av heisen
		e.Requests[e.Floor][elevio.BT_Cab] = 0

		switch e.Dirn {
		case elevio.MD_Up:
			if !RequestsAbove(e) {
				e.Requests[e.Floor][elevio.BT_HallUp] = 0
			}
			e.Requests[e.Floor][elevio.BT_HallDown] = 0

		case elevio.MD_Down:
			if !RequestsBelow(e) {
				e.Requests[e.Floor][elevio.BT_HallDown] = 0
			}
			e.Requests[e.Floor][elevio.BT_HallUp] = 0

		case elevio.MD_Stop:
			fallthrough
		default:
			e.Requests[e.Floor][elevio.BT_HallUp] = 0
			e.Requests[e.Floor][elevio.BT_HallDown] = 0
		}
	}
	return e
}
