package elevator

import (
	"Primary/elevator/config"
	"Primary/elevator/elevio"
	"fmt"
	"runtime"
)

// --------------- FINITE STATE MACHINE FUNCTIONS --------------- //

// --- LOCAL FUNCTIONS --- //

func fsm_onInitBetweenFloors(ele *elevio.Ele) {
	ele.SetMotorDirection(elevio.MD_Down)
	Elevators[ele.ID-1].Dirn = elevio.MD_Down
	Elevators[ele.ID-1].Behaviour = config.EB_Moving
}

func fsm_onRequestButtonPress(ele *elevio.Ele, btn_floor int, btn_type elevio.ButtonType) {
	pc, _, _, _ := runtime.Caller(0)
	functionName := runtime.FuncForPC(pc).Name()
	fmt.Printf("\n\n%s(%d, %s)\n", functionName, btn_floor, elevio.Elevio_button_toString(btn_type))

	switch Elevators[ele.ID-1].Behaviour {
	case config.EB_DoorOpen:
		if requests_shouldClearImmediatly(Elevators[ele.ID-1], btn_floor, btn_type) {
			timer_start(ele, Elevators[ele.ID-1].Config.DoorOpenDuration_s)
		} else {
			Elevators[ele.ID-1].Request[btn_floor][btn_type] = true
		}

	case config.EB_Moving:
		Elevators[ele.ID-1].Request[btn_floor][btn_type] = true

	case config.EB_Idle:
		Elevators[ele.ID-1].Request[btn_floor][btn_type] = true
		var pair DirnBehaviourPair = request_chooseDirection(Elevators[ele.ID-1])
		Elevators[ele.ID-1].Dirn = pair.dirn
		Elevators[ele.ID-1].Behaviour = pair.behaviour

		switch pair.behaviour {
		case config.EB_DoorOpen:
			ele.SetDoorOpenLamp(true)
			timer_start(ele, Elevators[ele.ID-1].Config.DoorOpenDuration_s)
			Elevators[ele.ID-1] = requests_clearAtCurrentFloor(Elevators[ele.ID-1])

		case config.EB_Moving:
			ele.SetMotorDirection(Elevators[ele.ID-1].Dirn)

		case config.EB_Idle:
			break

		}

	}
	elevator_setAllLights(ele, Elevators)
	fmt.Printf("\nNew state:\n")
}

func fsm_onFloorArrival(ele *elevio.Ele, newFloor int) {
	pc, _, _, _ := runtime.Caller(0)
	functionName := runtime.FuncForPC(pc).Name()
	fmt.Printf("\n\n%s(%d)\n", functionName, newFloor)
	Elevators[ele.ID-1].Floor = newFloor
	ele.SetFloorIndicator(Elevators[ele.ID-1].Floor)
	switch Elevators[ele.ID-1].Behaviour {
	case config.EB_Moving:
		if requests_shouldStop(Elevators[ele.ID-1]) {
			ele.SetMotorDirection(elevio.MD_Stop)
			ele.SetDoorOpenLamp(true)
			Elevators[ele.ID-1] = requests_clearAtCurrentFloor(Elevators[ele.ID-1])
			timer_start(ele, Elevators[ele.ID-1].Config.DoorOpenDuration_s)
			elevator_setAllLights(ele, Elevators)
			Elevators[ele.ID-1].Behaviour = config.EB_DoorOpen
		}

	default:
		break
	}
	fmt.Printf("\nNew state:\n")
}

func fsm_onDoorTimeout(ele *elevio.Ele) {
	pc, _, _, _ := runtime.Caller(0)
	functionName := runtime.FuncForPC(pc).Name()
	fmt.Printf("\n\n%s()\n", functionName)

	switch Elevators[ele.ID-1].Behaviour {
	case config.EB_DoorOpen:
		var pair DirnBehaviourPair = request_chooseDirection(Elevators[ele.ID-1])
		Elevators[ele.ID-1].Dirn = pair.dirn
		Elevators[ele.ID-1].Behaviour = pair.behaviour

		switch Elevators[ele.ID-1].Behaviour {
		case config.EB_DoorOpen:
			timer_start(ele, Elevators[ele.ID-1].Config.DoorOpenDuration_s)
			Elevators[ele.ID-1] = requests_clearAtCurrentFloor(Elevators[ele.ID-1])
			elevator_setAllLights(ele, Elevators)

		case config.EB_Moving, config.EB_Idle:
			ele.SetDoorOpenLamp(false)
			ele.SetMotorDirection(Elevators[ele.ID-1].Dirn)

		}

	default:
		break
	}
	fmt.Printf("\nNew state:\n")
}
