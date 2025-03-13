package request_assigner

import "math"

func assign_requests(inputData Input) map[string][][]bool {
	assignments := make(map[string][][]bool)
	for elevator := range inputData.States {
		assignments[elevator] = make([][]bool, len(inputData.HallRequests))
		for i := range assignments[elevator] {
			assignments[elevator][i] = []bool{false, false} // [up, down]
		}
	}

	// Assign requests to elevators
	for floor, hallRequest := range inputData.HallRequests {
		if hallRequest[0] {
			bestElevator := findBestElevator(inputData, floor, "up")
			if bestElevator != "" {
				assignments[bestElevator][floor][0] = true
			}
		}
		if hallRequest[1] {
			bestElevator := findBestElevator(inputData, floor, "down")
			if bestElevator != "" {
				assignments[bestElevator][floor][1] = true
			}
		}
	}

	return assignments
}

func findBestElevator(inputData Input, floor int, direction string) string {
	var bestElevator string
	bestScore := math.Inf(1) // Infinity

	for elevator, state := range inputData.States {
		var score float64
		if state.Behaviour == "idle" {
			score = float64(int(math.Abs(float64(state.Floor - floor))))
		} else if state.Direction == direction {
			if (direction == "up" && state.Floor <= floor) || (direction == "down" && state.Floor >= floor) {
				score = float64(int(math.Abs(float64(state.Floor - floor))))
			} else {
				score = float64(int(math.Abs(float64(state.Floor-floor)))) + 10 // Penalize turning around
			}
		} else {
			score = float64(int(math.Abs(float64(state.Floor-floor)))) + 20 // Penalize opposite direction
		}

		if score < bestScore {
			bestScore = score
			bestElevator = elevator
		}
	}

	return bestElevator
}
