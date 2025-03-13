package fsm

import (
	"elevatorsystem/constants"
	"elevatorsystem/single-elevator/Driver-go/elevio"
	"elevatorsystem/single-elevator/elevatorLogic"
	"elevatorsystem/single-elevator/timer"
)

var elevator elevatorLogic.Elevator

func init() {
	elevator = elevatorLogic.ElevatorUninitialized()
}

func GetElevator() *elevatorLogic.Elevator {
	return &elevator
}

func setAllLights(e *elevatorLogic.Elevator) {

	for f := 0; f < constants.NUM_FLOORS; f++ {
		for btn := 0; btn < constants.NUM_BUTTONS; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), f, e.Orders[f][btn])
		}
	}
}

func OnInitBetweenFloors(e *elevatorLogic.Elevator) {
	elevio.SetMotorDirection(elevio.MD_Down)
	e.Direction = elevio.MD_Down
	e.Behaviour = elevatorLogic.EB_Moving
}

func OnOrderButtonPress(e *elevatorLogic.Elevator, ButtonEvent elevio.ButtonEvent) {

	btnFloor := ButtonEvent.Floor
	btnType := ButtonEvent.Button

	switch elevator.Behaviour {
	case elevatorLogic.EB_DoorOpen:
		if elevatorLogic.OrdersShouldClearImmediately(elevator, ButtonEvent) {
			timer.TimerStart(elevator.Config.DoorOpenDuration_s)
		} else {
			elevator.Orders[btnFloor][btnType] = true
		}

	case elevatorLogic.EB_Moving:
		elevator.Orders[btnFloor][btnType] = true

	case elevatorLogic.EB_Idle:
		elevator.Orders[btnFloor][btnType] = true
		pair := elevatorLogic.OrderChooseDirection(elevator)
		elevator.Direction = pair.Direction
		elevator.Behaviour = pair.Behaviour
		switch pair.Behaviour {
		case elevatorLogic.EB_DoorOpen:
			elevio.SetDoorOpenLamp(true)
			timer.TimerStart(elevator.Config.DoorOpenDuration_s)
			elevator = elevatorLogic.OrdersClearAtCurrentFloor(elevator)

		case elevatorLogic.EB_Moving:
			elevio.SetMotorDirection(elevator.Direction)

		case elevatorLogic.EB_Idle:
			// Do nothing
		}
	}

	setAllLights(e)

}

func OnFloorArrival(e *elevatorLogic.Elevator, newFloor int) {

	elevator.LastKnownFloor = newFloor

	elevio.SetFloorIndicator(elevator.LastKnownFloor)

	switch elevator.Behaviour {
	case elevatorLogic.EB_Moving:
		if elevatorLogic.OrdersShouldStop(elevator) {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			elevator = elevatorLogic.OrdersClearAtCurrentFloor(elevator)
			timer.TimerStart(elevator.Config.DoorOpenDuration_s)
			setAllLights(e)
			elevator.Behaviour = elevatorLogic.EB_DoorOpen
		}
	default:
		// Do nothing
	}
}

func OnDoorTimeout(e *elevatorLogic.Elevator) {

	switch elevator.Behaviour {
	case elevatorLogic.EB_DoorOpen:
		pair := elevatorLogic.OrderChooseDirection(elevator)
		elevator.Direction = pair.Direction
		elevator.Behaviour = pair.Behaviour

		switch elevator.Behaviour {
		case elevatorLogic.EB_DoorOpen:
			timer.TimerStart(elevator.Config.DoorOpenDuration_s)
			elevator = elevatorLogic.OrdersClearAtCurrentFloor(elevator)
			setAllLights(e)
		case elevatorLogic.EB_Moving, elevatorLogic.EB_Idle:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevator.Direction)
		}
	default:
		// Do nothing
	}
}
