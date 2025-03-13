package fsm

import (
	. "elevator/elevio"
	"elevator/requests"
	"elevator/timer"
	"fmt"
)

func FsmOnInitBetweenFloors(elevator *Elevator, floor chan int) {
	SetMotorDirection(MD_Down)

	f := <-floor

	for f != 0 {
		f = <-floor
		SetFloorIndicator(f)
	}

	SetFloorIndicator(f)
	SetMotorDirection(MD_Stop)
	elevator.Dirn = MD_Stop
	elevator.Behaviour = EB_Idle
	elevator.Floor = 0
}

func IsElevatorUninitialized(e Elevator) bool {
	return e.Floor == -1 && e.Dirn == MD_Stop && e.Behaviour == EB_Idle
}

func SetAllLights(es *Elevator) {
	for floor := 0; floor < N_FLOORS; floor++ {
		for btn := 0; btn < N_BUTTONS; btn++ {
			SetButtonLamp(ButtonType(btn), floor, es.Requests[floor][btn])
		}
	}
}

func FsmOnRequestButtonPress(btn_floor int, btn_type ButtonType, elevator *Elevator) {

	switch elevator.Behaviour {
	case EB_DoorOpen:
		if requests.RequestsShouldClearImmediately(*elevator, btn_floor, btn_type) {
			timer.TimerStart()
		} else {
			elevator.Requests[btn_floor][btn_type] = true
		}

	case EB_Moving:
		elevator.Requests[btn_floor][btn_type] = true

	case EB_Idle:
		elevator.Requests[btn_floor][btn_type] = true
		pair := requests.RequestsChooseDirection(*elevator)
		elevator.Dirn = pair.Dirn
		elevator.Behaviour = pair.Behaviour
		switch pair.Behaviour {
		case EB_DoorOpen:
			SetDoorOpenLamp(true)
			timer.TimerStart()
			*elevator = requests.RequestsClearAtCurrentFloor(*elevator)

		case EB_Moving:
			SetMotorDirection(elevator.Dirn)

		case EB_Idle:
		}

	}

	SetAllLights(elevator)
}

func FsmOnFloorArrival(newFloor int, elevator *Elevator) {

	elevator.Floor = newFloor

	SetFloorIndicator(elevator.Floor)

	switch elevator.Behaviour {
	case EB_Moving:
		if requests.RequestsShouldStop(*elevator) {
			SetMotorDirection(MD_Stop)
			SetDoorOpenLamp(true)
			*elevator = requests.RequestsClearAtCurrentFloor(*elevator)
			timer.TimerStart()
			SetAllLights(elevator)
			elevator.Behaviour = EB_DoorOpen

		}

	default:

	}
}

func FsmOnDoorTimeout(elevator *Elevator) {

	fmt.Printf("Elevator behaviour: %d", elevator.Behaviour)
	switch elevator.Behaviour {
	case EB_DoorOpen:
		pair := requests.RequestsChooseDirection(*elevator)
		fmt.Printf("Pair direction: %d \n", pair.Dirn)
		fmt.Printf("Pair behaviour: %d \n", pair.Behaviour)

		elevator.Dirn = pair.Dirn
		elevator.Behaviour = pair.Behaviour

		switch elevator.Behaviour {
		case EB_DoorOpen:
			timer.TimerStart()
			*elevator = requests.RequestsClearAtCurrentFloor(*elevator)
			SetAllLights(elevator)

		case EB_Moving:
			SetDoorOpenLamp(false)
			SetMotorDirection(elevator.Dirn)
		case EB_Idle:
			SetDoorOpenLamp(false)
			SetMotorDirection(elevator.Dirn)

		}
	default:

	}
}
