package fsm

import (
	"Sanntid/elevio"

	"fmt"
)

type ElevatorState int

const (
	Idle ElevatorState = iota
	MovingBetweenFloors
	MovingPassingFloor
	DoorOpen
)

type ElevatorEvents struct {
	NewOrder       elevio.ButtonEvent
	ArrivedAtFloor int
	DoorTimeout    bool
}

type ElevatorDescision struct {
	NextState      ElevatorState
	ElevatorOutput ElevatorOutput
}

type ElevatorOutput struct {
	MotorDirection elevio.MotorDirection
	Door           bool
	ButtonLights   [4][3]bool
}

type ElevatorInput struct {
	PressedButtons [4][3]bool
	PrevFloor      int
}

func RequestsAbove(elevator ElevatorInput) bool {
	for i := elevator.PrevFloor + 1; i < 4; i++ {
		if elevator.PressedButtons[i][0] || elevator.PressedButtons[i][1] || elevator.PressedButtons[i][2] {
			return true
		}
	}
	return false
}

func RequestsBelow(elevator ElevatorInput) bool {
	for i := 0; i < elevator.PrevFloor; i++ {
		if elevator.PressedButtons[i][0] || elevator.PressedButtons[i][1] || elevator.PressedButtons[i][2] {
			return true
		}
	}
	return false
}

func HandleFloorReached(event int, storedInput ElevatorInput, storedOutput ElevatorOutput) ElevatorDescision {
	var nextState ElevatorState
	var nextOutput ElevatorOutput
	var QueueNotEmpty bool = false
	var numLights int = 0
	storedInput.PrevFloor = event
	for i := 0; i < 3; i++ {
		for j := 0; j < 4; j++ {
			if storedOutput.ButtonLights[j][i] {
				QueueNotEmpty = true
				numLights++
				break
			}
		}
	}
	if !QueueNotEmpty {
		nextState = DoorOpen
		nextOutput.MotorDirection = elevio.MD_Stop
		nextOutput.ButtonLights = storedInput.PressedButtons
		nextOutput.Door = true
		return ElevatorDescision{nextState, nextOutput}
	}
	
	caseDown := storedOutput.MotorDirection == elevio.MD_Down && (storedOutput.ButtonLights[event][1] || storedOutput.ButtonLights[event][2] || !RequestsBelow(storedInput))
	caseUp := storedOutput.MotorDirection == elevio.MD_Up && (storedOutput.ButtonLights[event][0] || storedOutput.ButtonLights[event][2] || !RequestsAbove(storedInput))

	if caseDown  {
		nextState = DoorOpen
		nextOutput.MotorDirection = elevio.MD_Stop
		nextOutput.Door = true
		nextOutput.ButtonLights = storedInput.PressedButtons
		if !RequestsBelow(storedInput) {
			nextOutput.ButtonLights[event][0] = false
		}
		nextOutput.ButtonLights[event][1] = false
		nextOutput.ButtonLights[event][2] = false
		storedInput.PressedButtons = storedOutput.ButtonLights
		return ElevatorDescision{nextState, nextOutput}
	}
	if  caseUp {
		nextState = DoorOpen
		nextOutput.MotorDirection = elevio.MD_Stop
		nextOutput.Door = true
		nextOutput.ButtonLights = storedInput.PressedButtons
		if !RequestsAbove(storedInput) {
			nextOutput.ButtonLights[event][1] = false
		}
		nextOutput.ButtonLights[event][0] = false
		nextOutput.ButtonLights[event][2] = false
		storedInput.PressedButtons = storedOutput.ButtonLights
		return ElevatorDescision{nextState, nextOutput}
	}
	return ElevatorDescision{MovingPassingFloor, storedOutput}
}

func HandleNewOrder(state ElevatorState, event elevio.ButtonEvent, storedInput ElevatorInput, storedOutput ElevatorOutput) ElevatorDescision {

	var nextState ElevatorState
	var nextOutput ElevatorOutput
	if state == Idle {
		fmt.Println(storedInput.PrevFloor, event.Floor)
		if storedInput.PrevFloor < event.Floor {
			nextOutput.MotorDirection = elevio.MD_Up
			nextOutput.ButtonLights = storedInput.PressedButtons
			nextState = MovingBetweenFloors

		} else if storedInput.PrevFloor > event.Floor {
			nextOutput.MotorDirection = elevio.MD_Down
			nextOutput.ButtonLights = storedInput.PressedButtons
			nextState = MovingBetweenFloors

		} else {
			nextOutput.MotorDirection = elevio.MD_Stop
			nextState = DoorOpen
			nextOutput.Door = true
			nextOutput.ButtonLights = storedInput.PressedButtons
			nextOutput.ButtonLights[event.Floor][0] = false
			nextOutput.ButtonLights[event.Floor][1] = false
			nextOutput.ButtonLights[event.Floor][2] = false
			storedInput.PressedButtons = nextOutput.ButtonLights

		}
		return ElevatorDescision{nextState, nextOutput}
	} else if state == DoorOpen {
		nextOutput.ButtonLights = storedInput.PressedButtons
		nextOutput.MotorDirection = elevio.MD_Stop

	} else {
		nextOutput.ButtonLights = storedInput.PressedButtons
		nextOutput.MotorDirection = storedOutput.MotorDirection
	}
	return ElevatorDescision{state, nextOutput}

}
func HandleDoorTimeout(storedInput ElevatorInput, storedOutput ElevatorOutput) ElevatorDescision {
	var nextState ElevatorState
	var nextOutput ElevatorOutput
	unservedOrders := false
	for i := 0; i < 4; i++ {
		if storedInput.PressedButtons[i][0] || storedInput.PressedButtons[i][1] || storedInput.PressedButtons[i][2] {
			unservedOrders = true
			break
		}
	}
	if !unservedOrders {
		nextOutput.Door = false
		nextState = Idle

	} else {
		switch storedOutput.MotorDirection {
		case elevio.MD_Stop:
			if RequestsAbove(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Up
				nextState = MovingBetweenFloors
			} else if RequestsBelow(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Down
				nextState = MovingBetweenFloors
			} else {
				nextOutput.MotorDirection = elevio.MD_Stop
				nextState = MovingBetweenFloors
			}
		case elevio.MD_Up:
			if RequestsAbove(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Up
				nextState = MovingBetweenFloors
			} else if RequestsBelow(storedInput) && (storedInput.PressedButtons[storedInput.PrevFloor][0]) {
				nextState = DoorOpen
				nextOutput.Door = true
				nextOutput.MotorDirection = elevio.MD_Stop
			} else if RequestsBelow(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Down
				nextState = MovingBetweenFloors
			} else {
				nextOutput.MotorDirection = elevio.MD_Stop
				nextState = MovingBetweenFloors
			}

		case elevio.MD_Down:
			if RequestsBelow(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Down
				nextState = MovingBetweenFloors
			} else if RequestsAbove(storedInput) && (storedInput.PressedButtons[storedInput.PrevFloor][1]) {
				nextState = DoorOpen
				nextOutput.Door = true
				nextOutput.MotorDirection = elevio.MD_Stop
			} else if RequestsAbove(storedInput) {
				nextOutput.MotorDirection = elevio.MD_Up
				nextState = MovingBetweenFloors
			} else {
				nextOutput.MotorDirection = elevio.MD_Stop
				nextState = MovingBetweenFloors
			}
		}

	}
	nextOutput.ButtonLights = storedOutput.ButtonLights
	return ElevatorDescision{nextState, nextOutput}
}
