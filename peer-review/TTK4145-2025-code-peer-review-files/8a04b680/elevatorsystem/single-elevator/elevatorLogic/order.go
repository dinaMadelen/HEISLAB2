package elevatorLogic

import (
	"elevatorsystem/single-elevator/Driver-go/elevio"
	"elevatorsystem/constants"
)

type DirectionBehaviourPair struct {
	Direction elevio.MotorDirection
	Behaviour ElevatorBehaviour
}

func OrdersAbove(e Elevator) bool {
	for f := e.LastKnownFloor + 1; f < constants.NUM_FLOORS; f++ {
		for btn := 0; btn < constants.NUM_BUTTONS; btn++ {
			if e.Orders[f][btn] {
				// print("Orders above returned true with LastKnownFloor: ", e.LastKnownFloor)
				return true
			}
		}
	}
	return false
}

func OrdersBelow(e Elevator) bool {
	for f := e.LastKnownFloor - 1; f >= 0; f-- {
		for btn := 0; btn < constants.NUM_BUTTONS; btn++ {
			if e.Orders[f][btn] {
				// print("Orders below returned true with LastKnownFloor: ", e.LastKnownFloor)
				return true
			}
		}
	}
	return false
}

func OrdersHere(e Elevator) bool {
	for btn := 0; btn < constants.NUM_BUTTONS; btn++ {
		if e.Orders[e.LastKnownFloor][btn] {
			return true
		}
	}
	return false
}
func OrderChooseDirection(e Elevator) DirectionBehaviourPair {
	switch e.Direction {
	case elevio.MD_Up:
		if OrdersAbove(e) {
			return DirectionBehaviourPair{elevio.MD_Up, EB_Moving}
		} else if OrdersHere(e) {
			return DirectionBehaviourPair{elevio.MD_Down, EB_DoorOpen}
		} else if OrdersBelow(e) {
			return DirectionBehaviourPair{elevio.MD_Down, EB_Moving}
		} else {
			return DirectionBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	case elevio.MD_Down:
		if OrdersBelow(e) {
			return DirectionBehaviourPair{elevio.MD_Down, EB_Moving}
		} else if OrdersHere(e) {
			return DirectionBehaviourPair{elevio.MD_Up, EB_DoorOpen}
		} else if OrdersAbove(e) {
			return DirectionBehaviourPair{elevio.MD_Up, EB_Moving}
		} else {
			return DirectionBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	case elevio.MD_Stop:
		if OrdersHere(e) {
			return DirectionBehaviourPair{elevio.MD_Stop, EB_DoorOpen}
		} else if OrdersAbove(e) {
			return DirectionBehaviourPair{elevio.MD_Up, EB_Moving}
		} else if OrdersBelow(e) {
			return DirectionBehaviourPair{elevio.MD_Down, EB_Moving}
		} else {
			return DirectionBehaviourPair{elevio.MD_Stop, EB_Idle}
		}
	default:
		return DirectionBehaviourPair{elevio.MD_Stop, EB_Idle}
	}
}

func OrdersShouldStop(e Elevator) bool {
	f := e.LastKnownFloor
	switch e.Direction {
	case elevio.MD_Down:
		return e.Orders[f][elevio.BT_HallDown] ||
			e.Orders[f][elevio.BT_Cab] ||
			!OrdersBelow(e)
	case elevio.MD_Up:
		return e.Orders[f][elevio.BT_HallUp] ||
			e.Orders[f][elevio.BT_Cab] ||
			!OrdersAbove(e)
	default:
		return true
	}
}

func OrdersShouldClearImmediately(e Elevator, ButtonEvent elevio.ButtonEvent) bool {
	switch e.Config.ClearOrderVariant {
	case CV_All:
		return e.LastKnownFloor == ButtonEvent.Floor
	case CV_InDirection:
		return (e.LastKnownFloor == ButtonEvent.Floor) &&
			((e.Direction == elevio.MD_Up && ButtonEvent.Button == elevio.BT_HallUp) ||
				(e.Direction == elevio.MD_Down && ButtonEvent.Button == elevio.BT_HallDown) ||
				(e.Direction == elevio.MD_Stop) ||
				(ButtonEvent.Button == elevio.BT_Cab))
	default:
		return false
	}
}

func OrdersClearAtCurrentFloor(e Elevator) Elevator {
	switch e.Config.ClearOrderVariant {
	case CV_All:
		for btn := 0; btn < constants.NUM_BUTTONS; btn++ {
			e.Orders[e.LastKnownFloor][btn] = false
		}
	case CV_InDirection:
		e.Orders[e.LastKnownFloor][elevio.BT_Cab] = false
		switch e.Direction {
		case elevio.MD_Up:
			if !OrdersAbove(e) && (!e.Orders[e.LastKnownFloor][elevio.BT_HallUp]) {
				e.Orders[e.LastKnownFloor][elevio.BT_HallDown] = false
			}
			e.Orders[e.LastKnownFloor][elevio.BT_HallUp] = false
		case elevio.MD_Down:
			if !OrdersBelow(e) && (!e.Orders[e.LastKnownFloor][elevio.BT_HallDown]) {
				e.Orders[e.LastKnownFloor][elevio.BT_HallUp] = false
			}
			e.Orders[e.LastKnownFloor][elevio.BT_HallDown] = false
		case elevio.MD_Stop:
			e.Orders[e.LastKnownFloor][elevio.BT_HallDown] = false
			e.Orders[e.LastKnownFloor][elevio.BT_HallUp] = false
		default:
			// Do nothing
		}
	default:
		break
	}
	return e
}
