package fsm

import (

	elev "github.com//HeisLab2025/elev_algo/elevator_io"
	"github.com//HeisLab2025/elev_algo/timer"
)

var elevator elev.Elevator
var outputDevice elev.ElevatorOutputDevice

//used by the nettverk module
func FetchElevatorStatus() elev.Elevator {
	return elevator
}

func Fsm_init() {
	elevator = elev.Elevator{}
	elevator.Config.DoorOpenDurationS = 3.0 // Default value
	elevator.Config.ClearRequestVariant = elev.CV_InDirn
	outputDevice = elev.Elevio_getOutputDevice()
	outputDevice.MotorDirection(0)
	elevator.Dirn = 0
	elevator.State = elev.IDLE
	elevator.Obs = false
}

func setAllLights(e elev.Elevator) {
	for floor := 0; floor < elev.N_FLOORS; floor++ {
		for btn := 0; btn < elev.N_BUTTONS; btn++ {
			outputDevice.RequestButtonLight(elev.ButtonType(btn), floor, e.Requests[floor][btn])
		}
	}
}

func Fsm_onInitBetweenFloors() {
	outputDevice.MotorDirection(-1)
	elevator.Dirn = -1
	elevator.State = elev.MOVE
}

func Fsm_onRequestButtonPress(btnFloor int, btnType int) {
	elevator.Requests[btnFloor][btnType] = true

	if btnType == 2 { //is cab-request
		Fsm_OrderInList(btnFloor, btnType)
	} else {
		setAllLights(elevator)
	}
}

//if the list from the HRA contains a true, the order must be taken by this elevator
//the action depends on the state the elevator is currently in
func Fsm_OrderInList(btnFloor int, btnType int) {
	elevator.OwnRequests[btnFloor][btnType] = true

	switch elevator.State {
	case elev.DOOROPEN:
		if requests_shouldClearImmediately(elevator, btnFloor, btnType) {
			elevator.OwnRequests[btnFloor][btnType] = false
			elevator.Requests[btnFloor][btnType] = false
			timer.Timer_start(elevator.Config.DoorOpenDurationS)

			Fsm_onDoorTimeout()
		} else {
			elevator.OwnRequests[btnFloor][btnType] = true
		}
	case elev.MOVE:
		elevator.OwnRequests[btnFloor][btnType] = true
	case elev.IDLE:
		elevator.OwnRequests[btnFloor][btnType] = true
		elevator.Dirn, elevator.State = requests_chooseDirection(elevator)

		switch elevator.State {
		case elev.DOOROPEN:
			outputDevice.DoorLight(true)
			timer.Timer_start(elevator.Config.DoorOpenDurationS)

			Fsm_onDoorTimeout()
			elevator = requests_clearAtCurrentFloor(elevator)
		case elev.MOVE:
			outputDevice.MotorDirection(elev.MotorDirection(elevator.Dirn))
		}
	}

	setAllLights(elevator)
}

func Fsm_onFloorArrival(newFloor int) {
	elevator.Floor = newFloor
	outputDevice.FloorIndicator(elevator.Floor)

	if elevator.State == elev.MOVE && requests_shouldStop(elevator) {
		outputDevice.MotorDirection(elev.MD_Stop)
		outputDevice.DoorLight(true)
		elevator = requests_clearAtCurrentFloor(elevator)
		timer.Timer_start(elevator.Config.DoorOpenDurationS)
		setAllLights(elevator)
		elevator.State = elev.DOOROPEN
	}
}

func Fsm_onDoorTimeout() {
	if elevator.State == elev.DOOROPEN {
		dirn, behaviour := requests_chooseDirection(elevator)
		elevator.Dirn = dirn
		elevator.State = behaviour

		switch elevator.State {
		case elev.DOOROPEN:
			timer.Timer_start(elevator.Config.DoorOpenDurationS)
			elevator = requests_clearAtCurrentFloor(elevator)
			setAllLights(elevator)
		case elev.MOVE, elev.IDLE:
			outputDevice.DoorLight(false)
			outputDevice.MotorDirection(elev.MotorDirection(elevator.Dirn))
		}
	}
}

func Fsm_stop() {
	elev.SetMotorDirection(elev.MD_Stop)
}

func Fsm_after_stop() {
	elev.SetMotorDirection(elevator.Dirn)
}

func GetObs() bool {
	return elevator.Obs
}

func FlipObs() {
	elevator.Obs = !elevator.Obs
}

// the functions below are used to handle requests in an effective manner

func requests_above(e elev.Elevator) bool {
	for f := e.Floor + 1; f < elev.N_FLOORS; f++ {
		for btn := 0; btn < elev.N_BUTTONS; btn++ {
			if e.OwnRequests[f][btn] {
				return true
			}
		}
	}
	return false
}

func requests_below(e elev.Elevator) bool {
	for f := 0; f < e.Floor; f++ {
		for btn := 0; btn < elev.N_BUTTONS; btn++ {
			if e.OwnRequests[f][btn] {
				return true
			}
		}
	}
	return false
}

func requests_here(e elev.Elevator) bool {
	for btn := 0; btn < elev.N_BUTTONS; btn++ {
		if e.OwnRequests[e.Floor][btn] {
			return true
		}
	}
	return false
}

func requests_chooseDirection(e elev.Elevator) (elev.MotorDirection, elev.State) {
	switch e.Dirn {
	case elev.MD_Up:
		if requests_above(e) {
			return elev.MD_Up, elev.MOVE
		} else if requests_here(e) {
			return elev.MD_Down, elev.DOOROPEN
		} else if requests_below(e) {
			return elev.MD_Down, elev.MOVE
		}
	case elev.MD_Down:
		if requests_below(e) {
			return elev.MD_Down, elev.MOVE
		} else if requests_here(e) {
			return elev.MD_Up, elev.DOOROPEN
		} else if requests_above(e) {
			return elev.MD_Up, elev.MOVE
		}
	case elev.MD_Stop:
		if requests_here(e) {
			return elev.MD_Stop, elev.DOOROPEN
		} else if requests_above(e) {
			return elev.MD_Up, elev.MOVE
		} else if requests_below(e) {
			return elev.MD_Down, elev.MOVE
		}
	}
	return elev.MD_Stop, elev.IDLE
}

func requests_shouldStop(e elev.Elevator) bool {
	switch e.Dirn {
	case elev.MD_Down:
		return e.OwnRequests[e.Floor][elev.B_HallDown] || e.OwnRequests[e.Floor][elev.B_Cab] || !requests_below(e)
	case elev.MD_Up:
		return e.OwnRequests[e.Floor][elev.B_HallUp] || e.OwnRequests[e.Floor][elev.B_Cab] || !requests_above(e)
	default:
		return true
	}

}

func requests_shouldClearImmediately(e elev.Elevator, btn_floor int, btn_type int) bool {
	switch e.Config.ClearRequestVariant {
	case elev.CV_All:
		return e.Floor == btn_floor
	case elev.CV_InDirn:
		return e.Floor == btn_floor &&
			(e.Dirn == elev.MD_Up && btn_type == elev.B_HallUp ||
				e.Dirn == elev.MD_Down && btn_type == elev.B_HallDown ||
				e.Dirn == elev.MD_Stop ||
				btn_type == elev.B_Cab)
	default:
		return false
	}
}

func requests_clearAtCurrentFloor(e elev.Elevator) elev.Elevator {
	switch e.Config.ClearRequestVariant {
	case elev.CV_All:
		for btn := 0; btn < elev.N_BUTTONS; btn++ {
			e.OwnRequests[e.Floor][btn] = false
		}
	case elev.CV_InDirn:
		e.OwnRequests[e.Floor][elev.B_Cab] = false
		e.Requests[e.Floor][elev.B_Cab] = false
		switch e.Dirn {
		case elev.MD_Up:
			if !requests_above(e) && !e.OwnRequests[e.Floor][elev.B_HallUp] {
				e.OwnRequests[e.Floor][elev.B_HallDown] = false
				e.Requests[e.Floor][elev.B_HallDown] = false
			}
			e.OwnRequests[e.Floor][elev.B_HallUp] = false
			e.Requests[e.Floor][elev.B_HallUp] = false

		case elev.MD_Down:
			if !requests_below(e) && !e.OwnRequests[e.Floor][elev.B_HallDown] {
				e.OwnRequests[e.Floor][elev.B_HallUp] = false
				e.Requests[e.Floor][elev.B_HallUp] = false
			}
			e.OwnRequests[e.Floor][elev.B_HallDown] = false
			e.Requests[e.Floor][elev.B_HallDown] = false
		case elev.MD_Stop:
			e.OwnRequests[e.Floor][elev.B_HallUp] = false
			e.Requests[e.Floor][elev.B_HallUp] = false
			e.OwnRequests[e.Floor][elev.B_HallDown] = false
			e.Requests[e.Floor][elev.B_HallDown] = false

		default:
			e.OwnRequests[e.Floor][elev.B_HallUp] = false
			e.Requests[e.Floor][elev.B_HallUp] = false
			e.OwnRequests[e.Floor][elev.B_HallDown] = false
			e.Requests[e.Floor][elev.B_HallDown] = false
		}
	}
	return e
}
