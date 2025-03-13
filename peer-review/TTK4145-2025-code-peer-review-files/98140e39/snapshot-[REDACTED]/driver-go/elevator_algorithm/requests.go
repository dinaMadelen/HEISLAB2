package elevatoralgorithm

func (e *Elevator) requestsAbove() bool {
	for f := e.Floor + 1; f < NumFloors; f++ {
		for btn := 0; btn < NumButtons; btn++ {
			if e.requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func (e *Elevator) requestsBelow() bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < NumButtons; btn++ {
			if e.requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func (e *Elevator) requestsHere() bool {
	for btn := 0; btn < NumButtons; btn++ {
		if e.requests[e.Floor][btn] {
			return true
		}
	}
	return false
}

func (e *Elevator) chooseDirection() behaviourPair {
	switch e.direction {
	case up:
		if e.requestsAbove() {
			return behaviourPair{up, moving}
		} else if e.requestsHere() {
			return behaviourPair{stop, doorOpen}
		} else if e.requestsBelow() {
			return behaviourPair{down, moving}
		} else {
			return behaviourPair{stop, idle}
		}
	case down:
		if e.requestsBelow() {
			return behaviourPair{down, moving}
		} else if e.requestsHere() {
			return behaviourPair{stop, doorOpen}
		} else if e.requestsAbove() {
			return behaviourPair{up, moving}
		} else {
			return behaviourPair{stop, idle}
		}
	case stop:
		if e.requestsHere() {
			return behaviourPair{stop, doorOpen}
		} else if e.requestsAbove() {
			return behaviourPair{up, moving}
		} else if e.requestsBelow() {
			return behaviourPair{down, moving}
		} else {
			return behaviourPair{stop, idle}
		}
	default:
		return behaviourPair{stop, idle}
	}
}

func (e *Elevator) shouldStop() bool {
	switch e.direction {
	case down:
		return e.requests[e.Floor][hallDown] || e.requests[e.Floor][cabButton] || !e.requestsBelow()
	case up:
		return e.requests[e.Floor][hallUp] || e.requests[e.Floor][cabButton] || !e.requestsAbove()
	default:
		return true
	}
}

func (e *Elevator) shouldClearImmediately(buttonFloor int, buttonType Button) bool {
	switch e.config.ClearRequestVariant {
	case clearAll:
		return e.Floor == buttonFloor
	case clearSameDir:
		return e.Floor == buttonFloor && ((e.direction == up && buttonType == hallUp) ||
			(e.direction == down && buttonType == hallDown) ||
			e.direction == stop ||
			buttonType == cabButton)
	default:
		return false
	}
}

func clearAtCurrentFloor(e Elevator) Elevator {
	switch e.config.ClearRequestVariant {
	case clearAll:
		for btn := 0; btn < NumButtons; btn++ {
			e.requests[e.Floor][btn] = false
		}

	case clearSameDir:
		e.requests[e.Floor][cabButton] = false
		switch e.direction {
		case up:
			if !e.requestsAbove() && !e.requests[e.Floor][hallUp] {
				e.requests[e.Floor][hallDown] = false
			}
			e.requests[e.Floor][hallUp] = false
		case down:
			if !e.requestsBelow() && !e.requests[e.Floor][hallDown] {
				e.requests[e.Floor][hallUp] = false
			}
			e.requests[e.Floor][hallDown] = false
		default:
			e.requests[e.Floor][hallUp] = false
			e.requests[e.Floor][hallDown] = false
		}
	}

	return e
}
