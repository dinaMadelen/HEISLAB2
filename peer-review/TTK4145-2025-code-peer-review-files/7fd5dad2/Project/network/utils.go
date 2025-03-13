package network

import (
	. "Project/dataenums"
)

func resetHallCalls() [NFloors][NButtons]ButtonState {
	var hallOrderList [NFloors][NButtons]ButtonState
	for floor := 0; floor < NFloors; floor++ {
		for button := 0; button < NButtons; button++ {
			hallOrderList[floor][button] = Initial
		}
	}
	return hallOrderList
}

func initializeElevatorList() [NElevators]HRAElevState {
	var list [NElevators]HRAElevState
	for i := 0; i < NElevators; i++ {
		list[i] = HRAElevState{
			Behavior:    "EBIdle",
			Floor:       0,
			Direction:   "MDStop",
			CabRequests: make([]bool, NFloors),
		}
	}
	return list
}
