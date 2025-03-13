package elevator

import (
	"ElevatorProject/elevio"
	"ElevatorProject/orderDistribution"
	//"fmt"
)

type ElevatorBehaviour int

const (
	NUMFLOORS     = 4
	NUMBUTTONTYPE = 3
	NUMELEVATORS  = 3
)

const (
	EB_Idle elevio.Behaviour = iota
	EB_DoorOpen
	EB_Moving
)

type ClearRequestVariant int

const (
	CV_All ClearRequestVariant = iota
	CV_InDirn
)

var (
	e orderDistribution.ElevatorState
	//outputDevice ElevOutputDevice
)

type ElevatorObject struct {
	Floor     int                           // Current floor
	Dirn      elevio.MotorDirection         // Elevator direction
	Requests  [NUMFLOORS][NUMBUTTONTYPE]int // Requests (two-dimentional array)
	Behaviour ElevatorBehaviour             // Elevators current behaviour
	Role      string

	Config struct { // Configure the elevator
		ClearRequestVariant ClearRequestVariant
		DoorOpenDurationS   float64
	}
}

func ElevatorUninitialized() ElevatorObject {
	return ElevatorObject{
		Floor:     -1,
		Dirn:      elevio.MD_Stop,
		Behaviour: ElevatorBehaviour(EB_Idle),
		Config: struct {
			ClearRequestVariant ClearRequestVariant
			DoorOpenDurationS   float64
		}{
			ClearRequestVariant: CV_All,
			DoorOpenDurationS:   3.0,
		},
		Requests: [NUMFLOORS][NUMBUTTONTYPE]int{}, // Initialise requests to zero
	}
}

/*func InitElevator(numFloors int, numButtonTypes int) {
	if numFloors > NUMFLOORS || numButtonTypes > NUMBUTTONTYPE {
		fmt.Println("Error: Configuration exceeds allowed array size.")
		return
	}

	e = orderDistribution.ElevatorState{
		Floor:     -1,                                     // Start outside a defined floor
		Dirn:      orderDistribution.DirnStop,             // The elevator starts as stopped
		Behaviour: orderDistribution.ElevatorBehaviour(0), // The elevator starts in "Idle" state
		Requests: ,
	}

	// Initialise the request matrix with zeros

	for i := 0; i < NUMFLOORS; i++ {
		for j := 0; j < NUMBUTTONTYPE; j++ {
			fmt.Println("Hei")
			e.Requests[i][j] = false
		}
	}
}
*/