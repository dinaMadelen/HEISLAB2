// Contains finite state machine helper functions for single elevator control.
package elev

import (
	"log/slog"

	"multivator/lib/driver-go/elevio"
	"multivator/src/config"
	"multivator/src/timer"
	"multivator/src/types"
)

// Handles button presses. In case of cab button, move elevator to floor and open door. In case of hall button, send hall call to network module.
func HandleCabOrder(elevator *types.ElevState, btn types.ButtonEvent, timerAction chan timer.TimerAction, hallOrderCh chan<- types.ButtonEvent, outMsgCh chan<- types.Message[types.Bid]) {
	if btn.Button == types.BT_Cab || elevio.GetFloor() == -1 {
		MoveElevator(elevator, btn, timerAction)
	} else if hallOrderCh != nil {
		hallOrderCh <- btn
	}
}

// Checks if elevator should stop at floor and opens door if so.
func HandleFloorArrival(elevator *types.ElevState, floor int, timerAction chan timer.TimerAction) {
	if floor == -1 {
		slog.Error("Floor arrival with undefined floor")
		return
	}
	elevator.Floor = floor
	elevio.SetFloorIndicator(floor)

	if shouldStop(elevator) {
		elevio.SetMotorDirection(types.MD_Stop)
		clearFloor(elevator)
		OpenDoor(elevator, timerAction)
	}
}

// Monitors obstruction state and stops elevator and door from closing if obstruction is detected.
func HandleObstruction(elevator *types.ElevState, obstruction bool, timerAction chan timer.TimerAction) {
	elevator.Obstructed = obstruction

	if obstruction {
		elevio.SetMotorDirection(types.MD_Stop)
		if elevio.GetFloor() != -1 {
			OpenDoor(elevator, timerAction)
		} else {
			elevator.Behaviour = types.Idle
		}
	} else {
		if elevator.Behaviour == types.DoorOpen {
			timerAction <- timer.Start
		} else {
			pair := chooseDirection(elevator)
			elevator.Dir = pair.Dir

			if pair.Behaviour == types.Moving {
				moveMotor(elevator)
			}
		}
	}
}

// Stops elevator and clears all orders and button lamps.
func HandleStop(elevator *types.ElevState) {
	elevio.SetMotorDirection(types.MD_Stop)
	elevio.SetDoorOpenLamp(false)
	for f := range config.NumFloors {
		for b := types.ButtonType(0); b < config.NumButtons; b++ {
			elevator.Orders[elevator.NodeID][f][b] = false
			elevio.SetButtonLamp(b, f, false)
		}
	}
}

// Handles door timeout with obstruction check.
func HandleDoorTimeout(elevator *types.ElevState, timerAction chan<- timer.TimerAction) {
	if elevator.Behaviour != types.DoorOpen {
		return
	}

	if elevator.Obstructed {
		timerAction <- timer.Start
		return
	}
	elevio.SetDoorOpenLamp(false)
	elevator.Behaviour = types.Idle
	clearFloor(elevator)

	pair := chooseDirection(elevator)
	elevator.Dir = pair.Dir

	if pair.Behaviour == types.Moving {
		moveMotor(elevator)
	}
}

// Open door, update state. Includes safety check to avoid opening door while moving.
func OpenDoor(elevator *types.ElevState, timerAction chan<- timer.TimerAction) {
	if elevio.GetFloor() == -1 {
		slog.Warn("Cannot open door while between floors")
		return
	}
	elevator.Behaviour = types.DoorOpen
	elevio.SetDoorOpenLamp(true)
	timerAction <- timer.Start
}
