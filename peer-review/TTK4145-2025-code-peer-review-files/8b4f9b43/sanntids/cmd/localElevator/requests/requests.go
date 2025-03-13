// File: cmd/localElevator/requests/requests.go

package requests

import (
    "sanntids/cmd/localElevator/elevator"
    "sanntids/cmd/localElevator/config"
	"Driver-go/elevio"
)

// Use elevator package types
type DirnBehaviourPair struct {
    MotorDirection      elevio.MotorDirection
    Behaviour elevator.ElevatorBehaviour
}

// Check if there are any requests above the current floor
func requests_above(e elevator.Elevator) bool {
    for f := e.Floor + 1; f < config.N_FLOORS; f++ {
        for btn := 0; btn < int(elevator.N_BUTTONS); btn++ {
            if e.Requests[f][btn] {
                return true
            }
        }
    }
    return false
}

// Check if there are any requests below the current floor
func requests_below(e elevator.Elevator) bool {
    for f := 0; f < e.Floor; f++ {
        for btn := 0; btn < int(elevator.N_BUTTONS); btn++ {
            if e.Requests[f][btn] {
                return true
            }
        }
    }
    return false
}

// Check if there is a request on the current floor
func requests_here(e elevator.Elevator) bool {
    for btn := 0; btn < int(elevator.N_BUTTONS); btn++ {
        if e.Requests[e.Floor][btn] {
            return true
        }
    }
    return false
}

// Decide which direction to move next, based on current requests and direction
func Requests_chooseDirection(e elevator.Elevator) DirnBehaviourPair {
    switch e.MotorDirection {
    case elevio.MD_Up:
        if requests_above(e) {
            return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
        } else if requests_here(e) {
            return DirnBehaviourPair{elevio.MD_Down, elevator.EB_DoorOpen}
        } else if requests_below(e) {
            return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
        } else {
            return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
        }

    case elevio.MD_Down:
        if requests_below(e) {
            return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
        } else if requests_here(e) {
            return DirnBehaviourPair{elevio.MD_Up, elevator.EB_DoorOpen}
        } else if requests_above(e) {
            return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
        } else {
            return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
        }

    case elevio.MD_Stop:
        fallthrough
    default:
        if requests_here(e) {
            return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_DoorOpen}
        } else if requests_above(e) {
            return DirnBehaviourPair{elevio.MD_Up, elevator.EB_Moving}
        } else if requests_below(e) {
            return DirnBehaviourPair{elevio.MD_Down, elevator.EB_Moving}
        } else {
            return DirnBehaviourPair{elevio.MD_Stop, elevator.EB_Idle}
        }
    }
}

// Determine if the elevator should stop at the current floor
func RequestsShouldStop(e elevator.Elevator) bool {
    switch e.MotorDirection {
    case elevio.MD_Down:
        return e.Requests[e.Floor][elevator.BT_HallDown] || 
               e.Requests[e.Floor][elevator.BT_Cab] || 
               !requests_below(e)

    case elevio.MD_Up:
        return e.Requests[e.Floor][elevator.BT_HallUp] || 
               e.Requests[e.Floor][elevator.BT_Cab] ||
               !requests_above(e)

    case elevio.MD_Stop:
        fallthrough
    default:
        return true
    }
}

// Decide if a request should be cleared immediately based on variant
func Requests_shouldClearImmediately(e elevator.Elevator, btnFloor int, btnType elevio.ButtonType) bool {
    switch e.Config.ClearRequestVariant {
    case config.CV_All:
        return e.Floor == btnFloor

    case config.CV_InDirn:
        return e.Floor == btnFloor &&
            ((e.MotorDirection == elevio.MD_Up && btnType == elevio.BT_HallUp) ||
                (e.MotorDirection == elevio.MD_Down && btnType == elevio.BT_HallDown) ||
                e.MotorDirection == elevio.MD_Stop ||
                btnType == elevio.BT_Cab)

    default:
        return false
    }
}

func Requests_clearAtCurrentFloor(e elevator.Elevator) elevator.Elevator {
    switch e.Config.ClearRequestVariant {
    case config.CV_All:
        for btn := 0; btn < int(elevator.N_BUTTONS); btn++ {
            e.Requests[e.Floor][btn] = false
        }

    case config.CV_InDirn:
        e.Requests[e.Floor][elevator.BT_Cab] = false

        switch e.MotorDirection {
        case elevio.MD_Up:
            e.Requests[e.Floor][elevator.BT_HallUp] = false

            if !requests_above(e) && !e.Requests[e.Floor][elevator.BT_HallUp] {
                e.Requests[e.Floor][elevator.BT_HallDown] = false
            }

        case elevio.MD_Down:
            e.Requests[e.Floor][elevator.BT_HallDown] = false
            if !requests_below(e) && !e.Requests[e.Floor][elevator.BT_HallDown] {
                e.Requests[e.Floor][elevator.BT_HallUp] = false
            }

        case elevio.MD_Stop:
            fallthrough
        default:
            e.Requests[e.Floor][elevator.BT_HallUp] = false
            e.Requests[e.Floor][elevator.BT_HallDown] = false
        }
    }
    return e
}

