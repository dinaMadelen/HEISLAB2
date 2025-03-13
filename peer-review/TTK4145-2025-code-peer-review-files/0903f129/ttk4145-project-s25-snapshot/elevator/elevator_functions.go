package elevator

import (
	"Project/config"
	"Project/elevator/elevio"
)

// Returns true if there are requests above the elevator's current floor
func requestsAbove(e Elevator) bool {
	for f := e.Floor + 1; f < N_FLOORS; f++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			if e.Queue[f][btn] {
				return true
			}
		}
	}
	return false
}

// Returns true if there are requests below the elevator's current floor
func requestsBelow(e Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			if e.Queue[f][btn] {
				return true
			}
		}
	}
	return false
}

// Returns true if there are requests at the elevator's current floor
func requestsHere(e Elevator) bool {
	for btn := 0; btn < N_BUTTONS; btn++ {
		if e.Queue[e.Floor][btn] {
			return true
		}
	}
	return false
}

func chooseDirection(e Elevator) (ElevatorDir, ElevatorState) {
	switch e.Dir {
	case Up:
		if requestsAbove(e) {
			return Up, Moving
		} else if requestsHere(e) {
			return Down, DoorOpen
		} else if requestsBelow(e) {
			return Down, Moving
		} else {
			return Stop, Idle
		}
	case Down:
		if requestsBelow(e) {
			return Down, Moving
		} else if requestsHere(e) {
			return Up, DoorOpen
		} else if requestsAbove(e) {
			return Up, Moving
		} else {
			return Stop, Idle
		}
	case Stop:
		if requestsHere(e) {
			return Stop, DoorOpen
		} else if requestsAbove(e) {
			return Up, Moving
		} else if requestsBelow(e) {
			return Down, Moving
		} else {
			return Stop, Idle
		}
	default:
		return Stop, Idle
	}

}

// Må se mer på en bedre løsning for denne
func shouldStop(e Elevator) bool {
	switch config.CLEAR_REQUEST_VARIANT {
	case config.All:
		return e.Queue[e.Floor][elevio.BT_HallUp] ||
			e.Queue[e.Floor][elevio.BT_HallDown] ||
			e.Queue[e.Floor][elevio.BT_Cab]
	case config.InDir:
		switch e.Dir {
		case Down:
			return e.Queue[e.Floor][elevio.BT_HallDown] || e.Queue[e.Floor][elevio.BT_Cab] || !requestsBelow(e)
		case Up:
			return e.Queue[e.Floor][elevio.BT_HallUp] || e.Queue[e.Floor][elevio.BT_Cab] || !requestsAbove(e)
		case Stop:
			return true
		default:
			return true
		}
	default:
	}
	return false

}

// Clears all requests at the elevator's current floor
// Could also pass cleared requests to a channel
func clearRequestsAtFloor(e *Elevator,

// orderServed chan<- OrderUpdate
) {
	switch config.CLEAR_REQUEST_VARIANT {
	case config.All:
		for btn := 0; btn < N_BUTTONS; btn++ {
			e.Queue[e.Floor][btn] = false
			//orderServed <- OrderUpdate{Floor: e.Floor, Button: ButtonType(elevio.ButtonType(btn)), Served: true}
		}
	case config.InDir:
		e.Queue[e.Floor][elevio.BT_Cab] = false
		switch e.Dir {
		case Up:
			if !requestsAbove(*e) && !e.Queue[e.Floor][elevio.BT_HallUp] {
				e.Queue[e.Floor][elevio.BT_HallDown] = false
				//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallDown, Served: true}
			}
			e.Queue[e.Floor][elevio.BT_HallUp] = false
			//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallUp, Served: true}
		case Down:
			if !requestsBelow(*e) && !e.Queue[e.Floor][elevio.BT_HallDown] {
				e.Queue[e.Floor][elevio.BT_HallUp] = false
				//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallUp, Served: true}
			}
			e.Queue[e.Floor][elevio.BT_HallDown] = false
			//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallDown, Served: true}
		case Stop:
			e.Queue[e.Floor][elevio.BT_HallUp] = false
			e.Queue[e.Floor][elevio.BT_HallDown] = false
			//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallUp, Served: true}
			//orderServed <- OrderUpdate{Floor: e.Floor, Button: BT_HallDown, Served: true}

		default:
		}
	default:
	}
}

// Sets all lights in the elevator to match the elevator's queue
func setAllLights(e Elevator) {
	for floor := 0; floor < N_FLOORS; floor++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, e.Queue[floor][btn])
		}
	}
}

// Returns true if the elevator should clear requests immediately when arriving at a floor
func shouldClearImmediately(e Elevator, floor int, button elevio.ButtonType) bool {
	switch config.CLEAR_REQUEST_VARIANT {
	case config.All:
		return e.Floor == floor
	case config.InDir:
		return e.Floor == floor && ((e.Dir == Up && button == elevio.BT_HallUp) ||
			(e.Dir == Down && button == elevio.BT_HallDown) ||
			e.Dir == Stop ||
			button == elevio.BT_Cab)
	default:
		return false
	}
}

// Initializes the elevator
func ElevatorInit() Elevator {
	return Elevator{
		Floor: elevio.GetFloor(),
		//Floor: 0,
		State: Idle,
		Dir:   Stop,
		Queue: [N_FLOORS][N_BUTTONS]bool{},
	}
}
