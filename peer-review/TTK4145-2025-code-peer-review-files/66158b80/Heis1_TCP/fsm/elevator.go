package fsm

import (
	"Driver-go/elevio"
	"fmt"
	"time"
)

// NumFloors and NumButtons are global variables
var NumFloors int = 4
var NumButtons int = 3

// Converting Dirn to MotorDirection
// To determine which direction the motor should physically rotate based on the planned direction.
func GetMotorDirectionFromDirn(dirn Dirn) elevio.MotorDirection {
	switch dirn {
	case D_Up:
		return elevio.MD_Up
	case D_Down:
		return elevio.MD_Down
	case D_Stop:
		return elevio.MD_Stop
	default:
		return elevio.MD_Stop
	}
}

// Direction type. up = 1, down = 0
type Dirn int

const (
	D_Up   Dirn = 1
	D_Down Dirn = -1
	D_Stop Dirn = 0
)

// ElevatorBehaviour type. Idle = 0, DoorOpen = 1, Moving = 2
type ElevatorBehaviour int

const (
	EB_Idle     ElevatorBehaviour = 0
	EB_DoorOpen ElevatorBehaviour = 1
	EB_Moving   ElevatorBehaviour = 2
)

// Elevator struct containing floor, moving direction and requests
// is used to keep track of the elevators state
// is basicly a elevator object
type Elevator struct {
	ID        int
	Floor     int
	Dirn      Dirn
	Behaviour ElevatorBehaviour
	//Buttons in hall and cab x=floor y=button
	Requests           [4][3]bool
	DoorOpenDuration_s float64
	Obstruction        bool
}

// Elevator initializer function
func NewElevator() Elevator {
	var elevator_setup Elevator = Elevator{
		ID:                 0,
		Floor:              -1,           // Uninitialized floor
		Dirn:               D_Stop,       // Not moving
		Behaviour:          EB_Idle,      // Idle state
		Requests:           [4][3]bool{}, // No requests initially
		DoorOpenDuration_s: 3.0,
		Obstruction:        false, // Default door open duration
	}

	return elevator_setup
}

func PrintElevator(elevator Elevator) {
	fmt.Printf("\n\nElevator:\n")
	fmt.Printf("ID: %d\n", elevator.ID)
	fmt.Printf("Floor: %d\n", elevator.Floor)
	fmt.Printf("Direction: %v\n", elevator.Dirn)
	fmt.Printf("Behaviour: %v\n", elevator.Behaviour)
	fmt.Printf("Requests: %v\n", elevator.Requests)
	fmt.Printf("Obstruction: %v\n", elevator.Obstruction)
}

// Function to set the obstruction status of the elevator
func SetObsructionStatus(status bool) {
	elevator.Obstruction = status
}

func SetElevatorToValidStartPossition() {
	fmt.Println("Elevator initialized")
	for {
		if elevio.GetFloor() == -1 {
			elevio.SetMotorDirection(elevio.MD_Down)
		} else {
			elevio.SetMotorDirection(elevio.MD_Stop)
			break
		}
		time.Sleep(_pollRate)

	}
}
