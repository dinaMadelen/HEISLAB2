package logic

import (
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/driver/elevio"
	"fmt"
	"time"
)

func ControlElevator(currentFloor int, currentDir *elevio.MotorDirection, elevator *config.Elevator, elevators *map[string]config.Elevator) {

	if ShouldStop(currentFloor, *currentDir, elevator.Requests) {
		elevio.SetMotorDirection(elevio.MD_Stop)
		//ClearRequestsAtFloor(currentFloor, *currentDir, elevator.Requests)

		UpdateButtonLights(elevator.Requests)

		elevio.SetDoorOpenLamp(true)
		time.Sleep(3 * time.Second)
		MarkRequestCompleted(elevator)
		ClearRequestsIfCompleted(elevators, elevator)
		elevio.SetDoorOpenLamp(false)

		*currentDir = ChooseDirection(elevator, *currentDir)
		elevio.SetMotorDirection(*currentDir)
	}

}

func UpdateButtonLights(requests [][]int) {
	for floor := 0; floor < len(requests); floor++ {
		for btn := 0; btn < 3; btn++ {
			if requests[floor][btn] == 2 {
				elevio.SetButtonLamp(elevio.ButtonType(btn), floor, true)
			}
			if requests[floor][btn] == 0 {
				elevio.SetButtonLamp(elevio.ButtonType(btn), floor, false)
			}
		}
	}
}

func ChooseDirection(elevator *config.Elevator, currentDir elevio.MotorDirection) elevio.MotorDirection {
	requests := elevator.Requests
	fmt.Printf("Current Requests: %v\n", requests)

	if currentDir == elevio.MD_Stop {
		for floor := 0; floor < config.NumFloors; floor++ {
			for btn := 0; btn < config.NumButtons; btn++ {
				if requests[floor][btn] == 2 {
					if floor > elevator.Floor {
						return elevio.MD_Up
					} else if floor < elevator.Floor {
						return elevio.MD_Down
					}
				}
			}
		}
	}

	if currentDir == elevio.MD_Up {
		if hasOrdersAbove(elevator.Floor, requests) {
			return elevio.MD_Up
		}
	} else if currentDir == elevio.MD_Down {
		if hasOrdersBelow(elevator.Floor, requests) {
			return elevio.MD_Down
		}
	}

	if hasOrdersAbove(elevator.Floor, requests) {
		return elevio.MD_Up
	}
	if hasOrdersBelow(elevator.Floor, requests) {
		return elevio.MD_Down
	}

	return elevio.MD_Stop
}

func ShouldStop(currentFloor int, currentDir elevio.MotorDirection, orders [][]int) bool {
	if orders[currentFloor][elevio.BT_Cab] == 2 {
		return true
	}
	if currentDir == elevio.MD_Up && orders[currentFloor][elevio.BT_HallUp] == 2 {
		return true
	}
	if currentDir == elevio.MD_Down && orders[currentFloor][elevio.BT_HallDown] == 2 {
		return true
	}
	if (currentDir == elevio.MD_Up && !hasOrdersAbove(currentFloor, orders)) ||
		(currentDir == elevio.MD_Down && !hasOrdersBelow(currentFloor, orders)) {
		return true
	}
	return false
}
func ClearRequestsIfCompleted(elevators *map[string]config.Elevator, elev *config.Elevator) {
	// If there is only one elevator, no need to check others.
	if len(*elevators) == 1 {
		for floor := 0; floor < config.NumFloors; floor++ {
			for button := 0; button < config.NumButtons; button++ {
				if elev.Requests[floor][button] == 3 {
					elev.Requests[floor][button] = 0
				}
			}
		}
		return
	}

	// Iterate over all floors and buttons
	for floor := 0; floor < config.NumFloors; floor++ {
		for button := 0; button < config.NumButtons; button++ {
			// Check if the current elevator has completed the request (3)
			if elev.Requests[floor][button] == 3 {
				allOthersHaveOne := true

				// Check all other elevators
				for id, otherElev := range *elevators {
					if id != elev.ID { // Skip the current elevator
						if otherElev.Requests[floor][button] != 1 {
							allOthersHaveOne = false
							break
						}
					}
				}

				// If all other elevators have 1, clear all requests at this floor/button
				if allOthersHaveOne {
					for id := range *elevators {
						(*elevators)[id].Requests[floor][button] = 0
					}
				}
			}
		}
	}
}
