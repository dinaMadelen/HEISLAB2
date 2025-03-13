package elevator

import (
	"fmt"
	sharedData "root/SharedData"
	"root/elevio"
	"runtime"
)

var (
	elevator Elevator
)

func MakeFsm() {
	elevator = MakeUninitializedelevator()
	//elevator parameters are set to default

	FsmOnInitBetweenFloors()
	//elevator is set to move down from the unknown start posistion
	//The elevator will now know what floor it is on, and will update its state accordingly
}

func GetElevatordata() sharedData.Elevator_data {
	return sharedData.Elevator_data{Behavior: EbToString(elevator.behaviour), Floor: elevator.floor, Direction: ElevioDirToString(elevator.direction), CabRequests: GetCabRequests(elevator.requests)}
}

func SetAllLights() {
	//Basically just takes the requests from the button presses and lights up the corresponding button lights
	requests := makeRequests(sharedData.GetsharedHallRequests(), GetCabRequests(elevator.requests))
	for floor := 0; floor < NUM_FLOORS; floor++ {
		for btn := 0; btn < NUM_BUTTONS; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, requests[floor][btn])
		}
	}
}

func FsmOnInitBetweenFloors() {
	elevio.SetMotorDirection(elevio.MD_Down)
	elevator.direction = DIR_DOWN
	elevator.behaviour = BEHAVIOUR_MOVING
}

func FsmOnRequestButtonPress(btn_floor int, btn_type Button) {
	//This is the important module in the FSM. Here button-presses are handled
	//and depending on the state of the elevator, the elevator will find the correct next behacior
	//communication with the elevator is done with runtime. instead of printf. like in the provided C program

	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()
	var update [3]int
	fmt.Printf("\n\n%s(%d, %s)\n", frame.Function, btn_floor, ElevioButtonToString(btn_type))
	elevator.print()

	switch elevator.behaviour {
	case BEHAVIOUR_DOOR_OPEN:
		if elevator.RequestsShouldClearImmediately(btn_floor, btn_type) {
			StartTimer()
		} else {
			elevator.requests[btn_floor][btn_type] = true
			update = [3]int{btn_floor, int(btn_type), 1}
			go Transmitt_update_and_update_localHallRequests(update, GetElevatordata())
		}
	case BEHAVIOUR_MOVING:
		elevator.requests[btn_floor][btn_type] = true
		update = [3]int{btn_floor, int(btn_type), 1}
		go Transmitt_update_and_update_localHallRequests(update, GetElevatordata())

	case BEHAVIOUR_IDLE:
		elevator.requests[btn_floor][btn_type] = true


		pair := elevator.RequestsChooseDirection()
		elevator.direction = pair.dir
		elevator.behaviour = pair.behaviour
		switch pair.behaviour {
		case BEHAVIOUR_DOOR_OPEN:
			elevio.SetDoorOpenLamp(true)
			StartTimer()
			elevator = RequestsClearAtCurrentFloor(elevator)

		case BEHAVIOUR_MOVING:
			elevio.SetMotorDirection(elevio.MotorDirection(elevator.direction))
		}
		update = [3]int{btn_floor, int(btn_type), 1}
		go Transmitt_update_and_update_localHallRequests(update, GetElevatordata())

	}

	fmt.Printf("\nNew state:\n")
	elevator.print()
}

func FsmOnFloorArrival(newFloor int) {
	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()

	fmt.Printf("\n\n%s(%d)\n", frame.Function, newFloor)
	elevator.print()

	elevator.floor = newFloor

	elevio.SetFloorIndicator(elevator.floor)

	switch elevator.behaviour {
	case BEHAVIOUR_MOVING:
		if elevator.RequestsShouldStop() {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			elevator = RequestsClearAtCurrentFloor(elevator)
			StartTimer()
			SetAllLights()
			elevator.behaviour = BEHAVIOUR_DOOR_OPEN
		}
	}

	fmt.Printf("\nNew state:\n")
	elevator.print()
}

func FsmOnDoorTimeout() {
	pc := make([]uintptr, 15)
	n := runtime.Callers(2, pc)
	frames := runtime.CallersFrames(pc[:n])
	frame, _ := frames.Next()

	fmt.Printf("\n\n%s()\n", frame.Function)
	elevator.print()

	switch elevator.behaviour {
	case BEHAVIOUR_DOOR_OPEN:
		pair := elevator.RequestsChooseDirection()
		elevator.direction = pair.dir
		elevator.behaviour = pair.behaviour

		switch elevator.behaviour {
		case BEHAVIOUR_DOOR_OPEN:
			StartTimer()
			elevator = RequestsClearAtCurrentFloor(elevator)
			SetAllLights()
		case BEHAVIOUR_MOVING, BEHAVIOUR_IDLE:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevio.MotorDirection(elevator.direction))
		}
	}

	fmt.Printf("\nNew state:\n")
	elevator.print()
}

var doorObstructed bool

func DoorObstructed() {
	doorObstructed = true
	if elevator.behaviour == BEHAVIOUR_DOOR_OPEN {
		StartTimer()
	}
}

func DoorUnobstructed() {
	doorObstructed = false
	if elevator.behaviour == BEHAVIOUR_DOOR_OPEN {
		StartTimer()
	}
}

func IsDoorObstructed() bool {
	return doorObstructed
}

func GetCabRequests(matrix [NUM_FLOORS][3]bool) []bool {
	var column []bool
	for i := 0; i < len(matrix); i++ {
		column = append(column, matrix[i][2])
	}
	return column
}

func GetHallRequests(matrix [NUM_FLOORS][3]bool) [][2]bool {
	var newMatrix [][2]bool

	// Extract columns 1 and 2 (index 0 and 1)
	for i := 0; i < len(matrix); i++ {
		newMatrix = append(newMatrix, [2]bool{matrix[i][0], matrix[i][1]})
	}
	return newMatrix
}



func makeRequests(HallRequests [][2]bool, GetCabRequests []bool) [NUM_FLOORS][3]bool {
    var result [NUM_FLOORS][3]bool

    for i := 0; i < NUM_FLOORS; i++ {
        result[i][0] = HallRequests[i][0]
        result[i][1] = HallRequests[i][1]
        result[i][2] = GetCabRequests[i]
    }
    return result
}

func GetElevator()Elevator{
	return elevator
}