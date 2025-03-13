package main

import (
	"OneElevator/elevio"
	"fmt"
)

type ElevatorBehaviour int

const (
	N_FLOORS  = 4
	N_BUTTONS = 3
)

const (
	EB_Idle ElevatorBehaviour = iota
	EB_DoorOpen
	EB_Moving
)

// ClearMode defines how requests are cleared
type ClearMode int

const (
	ClearAll         ClearMode = iota
	ClearDirectional ClearMode = iota
)

type Config struct {
	ClearMode        ClearMode
	DoorOpenDuration float64
}

type Elevator struct {
	Floor     int
	Dirn      elevio.MotorDirection
	Behaviour ElevatorBehaviour
	Requests  [N_FLOORS][N_BUTTONS]int //2D array
	Config    Config
}

// Hjelpefunksjon for prints
func (eb ElevatorBehaviour) String() string {
	switch eb {
	case EB_Idle:
		return "Idle"
	case EB_DoorOpen:
		return "Door Open"
	case EB_Moving:
		return "Moving"
	default:
		return "Unknown"
	}
}

func ElevatorUninitialized() Elevator {
	return Elevator{
		Floor:     -1,
		Dirn:      elevio.MD_Stop,
		Behaviour: EB_Idle,
		Config: Config{
			ClearMode:        ClearAll,
			DoorOpenDuration: 3.0,
		},
		Requests: [N_FLOORS][N_BUTTONS]int{}, //Initialiserer førespørsler til 0
	}
}

func initElevator(numFloors int, numButtons int) {
	if numFloors > N_FLOORS || numButtons > N_BUTTONS {
		fmt.Printf("Error: Configuration exceeds allowed array size")
		return
	}

	elevator = Elevator{
		Floor:     -1,             // Startes utenfor definerte etasjer
		Dirn:      elevio.MD_Stop, //Initialiseres i ro
		Behaviour: EB_Idle,        //Starter i IDLE
		Config: Config{
			ClearMode:        ClearAll,
			DoorOpenDuration: 3.0,
		},
	}

	// Init bestillingsmatrise med nuller
	for i := 0; i < N_FLOORS; i++ {
		for j := 0; j < N_BUTTONS; j++ {
			elevator.Requests[i][j] = 0
		}
	}

	fmt.Printf("Elevator initialized: %+v\n", elevator)
}
