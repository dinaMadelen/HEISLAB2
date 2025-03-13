package pba

import (
	"Sanntid/fsm"
	"math"
)

// min function to find the minimum value in an array
func min(arr []int) int {
	minVal := math.MaxInt64
	for _, value := range arr {
		if value < minVal {
			minVal = value
		}
	}
	return minVal
}

// indexOf function to find the index of the minimum value
func indexOf(arr []int, value int) int {
	for i, v := range arr {
		if v == value {
			return i
		}
	}
	return -1 // Return -1 if the value is not found
}

func AssignRequest(order fsm.Order, status [fsm.NFloors][fsm.NButtons][fsm.MElevators]bool) ([fsm.NFloors][fsm.NButtons][fsm.MElevators]bool, int) {
	numFloorsToIdle := 0
	numDoorOpensToIdle := 0
	prevOrderFloor := 0
	completeTimes := make([]int, fsm.MElevators) // Declare completeTimes as an array

	for k := 0; k < fsm.MElevators; k++ {
		numFloorsToIdle = 0
		numDoorOpensToIdle = 0
		prevOrderFloor = 0
		for i := 0; i < fsm.NFloors; i++ {
			for j := 0; j < fsm.NButtons; j++ {
				if status[i][j][k] {
					numDoorOpensToIdle++
					numFloorsToIdle += int(math.Abs(float64(i - prevOrderFloor)))
					prevOrderFloor = i
				}
			}
		}
		completeTimes[k] = 3*numDoorOpensToIdle + numFloorsToIdle

	}

	minTime := min(completeTimes)
	responsibleElevator := indexOf(completeTimes, minTime)
	status[order.ButtonEvent.Floor][order.ButtonEvent.Button][responsibleElevator] = true

	return status, responsibleElevator
}
