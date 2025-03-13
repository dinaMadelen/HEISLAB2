package localController

import (
	"Driver-go/elevator/types"
)

// DirnBehaviourPair holds a direction and behaviour pair.
type DirnBehaviourPair struct {
	Dirn      types.ElevDirection
	Behaviour types.ElevBehaviour
}

// Utility Functions

// RequestsAbove returns true if there is any request above the current floor.
func RequestsAbove(e types.Elevator) bool {
	for f := e.Floor + 1; f < types.N_FLOORS; f++ {
		for btn := 0; btn < types.N_BUTTONS; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

// RequestsBelow returns true if there is any request below the current floor.
func RequestsBelow(e types.Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < types.N_BUTTONS; btn++ {
			if e.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

// RequestsHere returns true if there is any request at the current floor.
func RequestsHere(e types.Elevator) bool {
	for btn := 0; btn < types.N_BUTTONS; btn++ {
		if e.Requests[e.Floor][btn] {
			return true
		}
	}
	return false
}

// RequestsChooseDirection examines the current elevator state and returns
// a DirnBehaviourPair indicating the next direction and behaviour.
func RequestsChooseDirection(e types.Elevator) DirnBehaviourPair {
	switch e.Direction {
	case types.ED_Up:
		if RequestsAbove(e) {
			return DirnBehaviourPair{types.ED_Up, types.EB_Moving}
		} else if RequestsHere(e) {
			return DirnBehaviourPair{types.ED_Down, types.EB_DoorOpen}
		} else if RequestsBelow(e) {
			return DirnBehaviourPair{types.ED_Down, types.EB_Moving}
		}
	case types.ED_Down:
		if RequestsBelow(e) {
			return DirnBehaviourPair{types.ED_Down, types.EB_Moving}
		} else if RequestsHere(e) {
			return DirnBehaviourPair{types.ED_Up, types.EB_DoorOpen}
		} else if RequestsAbove(e) {
			return DirnBehaviourPair{types.ED_Up, types.EB_Moving}
		}
	case types.ED_Stop:
		if RequestsHere(e) {
			return DirnBehaviourPair{types.ED_Stop, types.EB_DoorOpen}
		} else if RequestsAbove(e) {
			return DirnBehaviourPair{types.ED_Up, types.EB_Moving}
		} else if RequestsBelow(e) {
			return DirnBehaviourPair{types.ED_Down, types.EB_Moving}
		}
	}
	return DirnBehaviourPair{types.ED_Stop, types.EB_Idle}
}

// RequestsShouldStop returns true if the elevator should stop at its current floor.
func RequestsShouldStop(e types.Elevator) bool {
	switch e.Direction {
	case types.ED_Down:
		return e.Requests[e.Floor][types.BT_HallDown] ||
			e.Requests[e.Floor][types.BT_Cab] ||
			!RequestsBelow(e)
	case types.ED_Up:
		return e.Requests[e.Floor][types.BT_HallUp] ||
			e.Requests[e.Floor][types.BT_Cab] ||
			!RequestsAbove(e)
	case types.ED_Stop:
		fallthrough
	default:
		return true
	}
}

// RequestsClearAtCurrentFloor clears requests at the current floor based on the configured clearing behavior.
func RequestsClearAtCurrentFloor(e types.Elevator) types.Elevator {
	switch e.Config.ClearRequestVariant {
	case types.CV_All:
		for btn := 0; btn < types.N_BUTTONS; btn++ {
			e.Requests[e.Floor][btn] = false
		}
	case types.CV_InDirn:
		e.Requests[e.Floor][types.BT_Cab] = false
		switch e.Direction {
		case types.ED_Up:
			if !RequestsAbove(e) && !e.Requests[e.Floor][types.BT_HallUp] {
				e.Requests[e.Floor][types.BT_HallDown] = false
			}
			e.Requests[e.Floor][types.BT_HallUp] = false
		case types.ED_Down:
			if !RequestsBelow(e) && !e.Requests[e.Floor][types.BT_HallDown] {
				e.Requests[e.Floor][types.BT_HallUp] = false
			}
			e.Requests[e.Floor][types.BT_HallDown] = false
		case types.ED_Stop:
			fallthrough
		default:
			e.Requests[e.Floor][types.BT_HallUp] = false
			e.Requests[e.Floor][types.BT_HallDown] = false
		}
	}
	return e
}

// FSM Functions

// FsmOnRequestButtonPress handles a new button press event by marking the request
// and, if the elevator is idle, deciding the next direction and behaviour.
func OnRequestButtonPress(e types.Elevator, floor int, btn int) types.Elevator {
	// Mark the new request.
	e.Requests[floor][btn] = true
	// If the elevator is idle, choose a new direction and behaviour.
	if e.Behaviour == types.EB_Idle {
		pair := RequestsChooseDirection(e)
		e.Direction = pair.Dirn
		e.Behaviour = pair.Behaviour
	}
	return e
}

// FsmOnFloorArrival handles a floor sensor event. It updates the current floor,
// and if the elevator is moving and should stop, changes state to door open.
func OnFloorArrival(e types.Elevator, floor int) types.Elevator {
	e.Floor = floor
	if e.Behaviour == types.EB_Moving && RequestsShouldStop(e) {
		// Stop the elevator and open the door.
		e.Behaviour = types.EB_DoorOpen
		// Clear requests at this floor.
		e = RequestsClearAtCurrentFloor(e)
	}
	return e
}

// FsmOnObstruction handles an obstruction event. Depending on your design,
// additional logic could be added here (for instance, pausing or restarting door timers).
func OnObstruction(e types.Elevator, obstructed bool) types.Elevator {
	// In this basic implementation, we simply return the state.
	// (The caller should handle timer adjustments based on obstruction status.)
	return e
}

// FsmOnStopPressed handles a stop button event by clearing all requests
// and switching the elevator state to door open.
func OnStopPressed(e types.Elevator) types.Elevator {
	for i := 0; i < types.N_FLOORS; i++ {
		for j := 0; j < types.N_BUTTONS; j++ {
			e.Requests[i][j] = false
		}
	}
	e.Behaviour = types.EB_DoorOpen
	return e
}

// FsmOnDoorTimeout handles the door timeout event by clearing requests at the current floor
// and choosing the next direction and behaviour.
func OnDoorTimeout(e types.Elevator) types.Elevator {
	e = RequestsClearAtCurrentFloor(e)
	pair := RequestsChooseDirection(e)
	e.Direction = pair.Dirn
	e.Behaviour = pair.Behaviour
	return e
}

// FsmOnMobilityTimeout handles the mobility timeout event, for example by stopping the elevator.
func OnMobilityTimeout(e types.Elevator) types.Elevator {
	e.Behaviour = types.EB_Idle
	return e
}
