package elevator

// DirnBehaviourPair strukturen holder retningen og oppførselen for heisen
type DirnBehaviourPair struct {
	Dirn      Dirn
	Behaviour ElevatorBehaviour
}

// requestsShouldStop sjekker om heisen skal stoppe på nåværende etasje
func requestsShouldStop(e Elevator, hallRequests [][2]bool) bool {

	// 1. Stopp alltid hvis en cab-request er på denne etasjen
	if e.CabRequests[e.Floor] {
		return true
	}

	// 2. Stopp hvis hall-forespørsel er tildelt denne heisen i denne etasjen
	if e.Dirn == D_Up && hallRequests[e.Floor][B_HallUp] {
		return true
	}
	if e.Dirn == D_Down && hallRequests[e.Floor][B_HallDown] {
		return true
	}

	// 3. Hvis det ikke er flere hall requests i denne retningen, stopp
	if e.Dirn == D_Up {
		for floor := e.Floor + 1; floor < N_FLOORS; floor++ {
			if hallRequests[floor][B_HallUp] || hallRequests[floor][B_HallDown] {
				return false // Fortsett å bevege seg
			}
		}
		return true // Stopp siden ingen flere hall requests i denne retningen
	}

	if e.Dirn == D_Down {
		for floor := 0; floor < e.Floor; floor++ {
			if hallRequests[floor][B_HallUp] || hallRequests[floor][B_HallDown] {
				return false // Fortsett å bevege seg
			}
		}
		return true // Stopp siden ingen flere hall requests i denne retningen
	}

	return false
}

// requestsClearAtCurrentFloor rydder bestillinger på gjeldende etasje
func requestsClearAtCurrentFloor(e Elevator) Elevator {
	// Clear cab request at this floor
	e.CabRequests[e.Floor] = false

	// Hall requests are assigned by hall_request_assigner, no need to clear them locally.

	return e
}

func hasPendingRequests(e Elevator, hallRequests [][2]bool) bool {

	// Sjekk cab requests
	for _, request := range e.CabRequests {
		if request {
			return true
		}
	}

	// Sjekk hall requests
	for floor := 0; floor < N_FLOORS; floor++ {
		if hallRequests[floor][B_HallUp] || hallRequests[floor][B_HallDown] {
			return true
		}
	}

	return false
}

// requestsChooseDirection velger retning basert på forespørsler
func requestsChooseDirection(e Elevator) Dirn {
	sharedState := GetSharedState()
	hallRequests := sharedState.HallRequests

	// Hvis det er noen forespørsler over heisen
	for f := e.Floor + 1; f < N_FLOORS; f++ {
		if hallRequests[f][B_HallUp] || hallRequests[f][B_HallDown] || e.CabRequests[f] {
			return D_Up
		}
	}

	// Hvis det er noen forespørsler under heisen
	for f := 0; f < e.Floor; f++ {
		if hallRequests[f][B_HallUp] || hallRequests[f][B_HallDown] || e.CabRequests[f] {
			return D_Down
		}
	}

	// Ingen forespørsler, stopp
	return D_Stop
}
