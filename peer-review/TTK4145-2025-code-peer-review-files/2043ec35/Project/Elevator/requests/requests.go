package requests

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/elevio"
	"elevproj/config"
)

type DirnBehaviourPair struct {
	Dirn      elevio.MotorDirection
	Behaviour elevator.ElevatorBehaviour
}

func ClearAtCurrentFloor(e elevator.Elevator) elevator.Elevator {
	newElevator := elevator.DeepCopyElevator(e)
	newElevator.Requests[newElevator.LatestFloor][elevio.BT_Cab] = false
	switch newElevator.Dirn {
	case elevio.MD_Up:
		if !requestsAbove(newElevator) && !newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallUp] {
			newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallDown] = false
		}
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallUp] = false

	case elevio.MD_Down:
		if !requestsBelow(newElevator) && !newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallDown] {
			newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallUp] = false
		}
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallDown] = false

	case elevio.MD_Stop:
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallUp] = false
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallDown] = false
	default:
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallUp] = false
		newElevator.Requests[newElevator.LatestFloor][elevio.BT_HallDown] = false

	}

	elevator.SetAllLights(newElevator)
	return newElevator
}

func requestsBelow(elev elevator.Elevator) bool {
	for f := 0; f < elev.LatestFloor; f++ {
		for btn := 0; btn < config.N_buttons; btn++ {
			if elev.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func requestsAbove(elev elevator.Elevator) bool {
	for f := elev.LatestFloor + 1; f < config.N_floors; f++ {
		for btn := 0; btn < config.N_buttons; btn++ {
			if elev.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func request_here(elev elevator.Elevator) bool {
	for btn := 0; btn < config.N_buttons; btn++ {
		if elev.Requests[elev.LatestFloor][btn] {
			return true
		}
	}
	return false
}

func ChooseDirection(elev elevator.Elevator) DirnBehaviourPair {
	switch elev.Dirn {
	case elevio.MD_Up:
		if requestsAbove(elev) {
			println("still up")
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_moving}
		} else if request_here(elev) {
			println("up to stop")
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_dooropen}
		} else if requestsBelow(elev) {
			println("up to down")
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_moving}
		} else {
			println("up to what")
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_idle}
		}

	case elevio.MD_Down:
		if requestsBelow(elev) {
			println("still down")
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_moving}
		} else if request_here(elev) {
			println("down to stop")
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_dooropen}
		} else if requestsAbove(elev) {
			println("down to up")
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_moving}
		} else {
			println("down to what")
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_idle}
		}

	case elevio.MD_Stop:
		if request_here(elev) {
			println("keep on here")
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_dooropen}
		} else if requestsAbove(elev) && elev.LatestFloor != config.N_floors-1 {
			println("start up")
			return DirnBehaviourPair{elevio.MD_Up, elevator.EB_moving}
		} else if requestsBelow(elev) {
			println("start down")
			return DirnBehaviourPair{elevio.MD_Down, elevator.EB_moving}
		} else {
			println("stop to what")
			return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_idle}
		}

	default:
		return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_idle}
	}
}

func ShouldStop(elev elevator.Elevator) bool {
	switch elev.Dirn {
	case elevio.MD_Down:
		return (elev.Requests[elev.LatestFloor][elevio.BT_HallDown]) || (elev.Requests[elev.LatestFloor][elevio.BT_Cab]) || !requestsBelow(elev)

	case elevio.MD_Up:
		return (elev.Requests[elev.LatestFloor][elevio.BT_HallUp]) || (elev.Requests[elev.LatestFloor][elevio.BT_Cab]) || !requestsAbove(elev)

	default:
		return true
	}
}

// burde denne forbedres?
func ShouldClearImmediately(elev elevator.Elevator, btnFloor int, btnType elevio.ButtonType) bool {
	return elev.LatestFloor == btnFloor && ((elev.Dirn == elevio.MD_Up && btnType == elevio.BT_HallUp) || (elev.Dirn == elevio.MD_Down && btnType == elevio.BT_HallDown) || elev.Dirn == elevio.MD_Stop || btnType == elevio.BT_Cab)

}
