package elevatoralgorithm

import (
	"elevatorproject/driver-go/elevator_algorithm/timer"
	"elevatorproject/driver-go/elevio"
	"fmt"
	"runtime"
	"time"
)

var (
	RunningElevator Elevator
)

func InitFsm() {
	RunningElevator = MakeUninitializedelevator()
	initBetweenFloors()
}

func setAllLights(elevator Elevator) {
	for floor := 0; floor < NumFloors; floor++ {
		for btn := 0; btn < NumButtons; btn++ {
			if floor == 4 {
				fmt.Println("floor ", floor)
				fmt.Println("button ", btn)
			}
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, elevator.requests[floor][btn])
		}
	}
}

func initBetweenFloors() {
	elevio.SetMotorDirection(elevio.MD_Down)
	RunningElevator.direction = down
	RunningElevator.behaviour = moving
}

func RequestButtonPressed(buttonFloor int, buttonType Button) {
	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()

	fmt.Printf("\n\n%s(%d, %s)\n", frame.Function, buttonFloor, buttonToString(buttonType))
	RunningElevator.PrintElevator()

	switch RunningElevator.behaviour {
	case doorOpen:
		if RunningElevator.shouldClearImmediately(buttonFloor, buttonType) {
			timer.StartTimer()
		} else {
			RunningElevator.requests[buttonFloor][buttonType] = true
		}
	case moving:
		RunningElevator.requests[buttonFloor][buttonType] = true
	case idle:
		RunningElevator.requests[buttonFloor][buttonType] = true
		pair := RunningElevator.chooseDirection()
		RunningElevator.direction = pair.dir
		RunningElevator.behaviour = pair.behaviour
		switch pair.behaviour {
		case doorOpen:
			elevio.SetDoorOpenLamp(true)
			timer.StartTimer()
			RunningElevator = clearAtCurrentFloor(RunningElevator)
		case moving:
			elevio.SetMotorDirection(elevio.MotorDirection(RunningElevator.direction))
		}
	}

	setAllLights(RunningElevator)

	fmt.Printf("\nNew state:\n")
	RunningElevator.PrintElevator()
}

func OnFloorArrival(newFloor int) {
	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()

	fmt.Printf("\n\n%s(%d)\n", frame.Function, newFloor)
	RunningElevator.PrintElevator()

	RunningElevator.Floor = newFloor

	elevio.SetFloorIndicator(RunningElevator.Floor)

	switch RunningElevator.behaviour {
	case moving:
		if RunningElevator.shouldStop() {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			RunningElevator = clearAtCurrentFloor(RunningElevator)
			timer.StartTimer()
			setAllLights(RunningElevator)
			RunningElevator.behaviour = doorOpen
		}
	}

	fmt.Printf("\nNew state:\n")
	RunningElevator.PrintElevator()
}

func OnDoorTimeout() {
	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()

	fmt.Printf("\n\n%s()\n", frame.Function)
	RunningElevator.PrintElevator()

	switch RunningElevator.behaviour {
	case doorOpen:
		pair := RunningElevator.chooseDirection()
		RunningElevator.direction = pair.dir
		RunningElevator.behaviour = pair.behaviour

		switch RunningElevator.behaviour {
		case doorOpen:
			timer.StartTimer()
			RunningElevator = clearAtCurrentFloor(RunningElevator)
			setAllLights(RunningElevator)
		case moving, idle:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevio.MotorDirection(RunningElevator.direction))
		}
	}

	fmt.Printf("\nNew state:\n")
	RunningElevator.PrintElevator()
}

func DoorObstructed() {
	if RunningElevator.behaviour == doorOpen {
		timer.StartTimer()
	}
}

func GetTimeout() time.Duration {
	return RunningElevator.config.DoorOpenDuration
}
