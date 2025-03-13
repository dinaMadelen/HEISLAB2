package main

import (
	"fmt"
	"time"
)

func main() {
	fmt.Println("Started!")

	// Init network and elevator

	inputPollRate := 25 * time.Millisecond
	input := elevioGetInputDevice()

	hardwareInit()

	if input.floorSensor() == -1 {
		fsmOnInitBetweenFloors()
	}

	prevRequests := make([][]int, N_FLOORS)
	for i := range prevRequests {
		prevRequests[i] = make([]int, N_BUTTONS)
	}

	prevFloor := -1
	prevObstr := 0

	for {
		for f := 0; f < N_FLOORS; f++ {
			for b := 0; b < N_BUTTONS; b++ {
				v := input.requestButton(f, ButtonType(b))
				if v != 0 && v != prevRequests[f][b] {
					fsmOnRequestButtonPress(f, ButtonType(b))
				}
				prevRequests[f][b] = v
			}
		}

		// Obstruction handling
		obstr := input.obstruction()
		if obstr != 0 && prevObstr == 0 {
			fsmOnObstruction()
		} else if obstr == 0 && prevObstr != 0 {
			fsmOnObstructionCleared()
		}
		prevObstr = obstr

		// Floor sensor handling
		floor := input.floorSensor()
		if floor != -1 && floor != prevFloor {
			fsmOnFloorArrival(floor)
		}
		prevFloor = floor

		// Timer handling
		if timerTimedOut() {
			timerStop()
			fsmOnDoorTimeout()
		}

		time.Sleep(inputPollRate)
	}
}