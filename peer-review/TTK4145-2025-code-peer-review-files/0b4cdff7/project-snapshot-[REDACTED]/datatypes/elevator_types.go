package datatypes

import (
	"project/elevio"
	"sync"
	"time"
)

const N_FLOORS = 4
const N_BUTTONS = 3
const N_HALL_BUTTONS = 2

type ElevBehaviour int

const (
	Idle     ElevBehaviour = 0
	Moving   ElevBehaviour = 1
	DoorOpen ElevBehaviour = 2
)

type Elevator struct {
	CurrentFloor int
	Direction    elevio.MotorDirection
	State        ElevBehaviour
	Orders       [N_FLOORS][N_BUTTONS]bool
	Config       ElevatorConfig
	StopActive   bool
}

type NetElevator struct {
	ID           string
	CurrentFloor int
	Direction    elevio.MotorDirection
	State        ElevBehaviour
	Orders       [4][3]bool
	Config       ElevatorConfig
	StopActive   bool
}

type ElevSharedInfo struct {
	Available    bool
	Behaviour    ElevBehaviour
	Direction    elevio.MotorDirection
	CurrentFloor int
	Mutex        sync.Mutex
}

type ElevatorContext struct {
	HallRequests     [N_FLOORS][N_HALL_BUTTONS]RequestType
	AllCabRequests   map[string][N_FLOORS]RequestType
	UpdatedInfoElevs map[string]ElevatorInfo
	PeerList         []string
	LocalID          string
}

type ElevatorConfig struct {
	DoorOpenDuration time.Duration
}
