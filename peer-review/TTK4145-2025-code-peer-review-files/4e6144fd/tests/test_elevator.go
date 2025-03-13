package tests

import (
	"fmt"
	"time"

	elevator "Driver-Elevator"
	elevio "Driver-Elevio"
)

// I Go blir init()-funksjoner i en pakke automatisk kjørt før main() starter.
// &
// Go only allows functions and variables to be accessed from outside their package if they are exported,
// meaning their names must start with an uppercase letter.
func Test_Elevator() {
	fmt.Println("Started!")

	// Load configuration
	config := elevator.LoadConfig("elevator/elevator.con")
	inputPollRateMs := config.InputPollRateMs

	elevio.Init("localhost:15657", elevator.N_FLOORS)

	// Initialize elevator IO
	inputDevice := elevio.GetInputDevice()

	// If the elevator starts between floors, handle initialization
	if inputDevice.FloorSensor() == -1 {
		elevator.FSMOnInitBetweenFloors()
	}

	// Declared outside the loop since static doesn't exist in Go for local variables
	var prev [elevator.N_FLOORS][elevator.N_BUTTONS]int // Tracks previous button states
	prevFloor := -1

	// Main loop
	for {
		// Sjekker fortløpende om knapper er trykket og oppdaterer systemet deretter.
		for f := 0; f < elevator.N_FLOORS; f++ {
			for b := 0; b < elevator.N_BUTTONS; b++ {
				v := inputDevice.RequestButton(elevio.ButtonType(b), f)
				if v && prev[f][b] == 0 {
					elevator.FSMOnRequestButtonPress(f, elevio.ButtonType(b))
				}
				prev[f][b] = boolToInt(v)
			}
		}

		// Floor sensor handling
		f := inputDevice.FloorSensor()
		if f != -1 && f != prevFloor {
			elevator.FSMOnFloorArrival(f)
		}
		prevFloor = f

		// Timer handling
		if elevator.TimerTimedOut() {
			elevator.TimerStop()
			elevator.FSMOnDoorTimeout()
		}
		// Sleep for input poll rate
		time.Sleep(time.Duration(inputPollRateMs) * time.Millisecond)
	}
}

func boolToInt(b bool) int {
	if b {
		return 1
	}
	return 0
}
