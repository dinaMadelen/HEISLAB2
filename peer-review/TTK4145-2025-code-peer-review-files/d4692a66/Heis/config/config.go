package config

import (
	"G19_heis2/Heis/driver/elevio"
	"G19_heis2/Heis/network/localip"
	"flag"
	"os"
	"fmt"
	"sync"
)

const (
	NumButtons = 3
	NumFloors  = 4
)

type ElevatorState int

const (
	IDLE ElevatorState = iota
	MOVING
	DOOR_OPEN
	STOPPED
)

type HRAElevState struct {
    Behavior    string      `json:"behaviour"`
    Floor       int         `json:"floor"` 
    Direction   string      `json:"direction"`
    CabRequests []bool      `json:"cabRequests"`
}


type HRAInput struct {
    HallRequests    [][2]bool                   `json:"hallRequests"`
    States          map[string]HRAElevState     `json:"states"`
}

var GlobalState = make(map[string]Elevator) 
var StateMutex sync.RWMutex

type Elevator struct {
	ID string
	Floor int
	CurrDirn elevio.MotorDirection
	Requests [][]int //0, 1, 2. Sitter i unconfiremd helt til confirmed har fått tatt den da blir det ned til 0 igjen. 
	State ElevatorState 
	IsOnline bool
	Timestamp int64
}

type NetworkChannels struct {
	StateRX chan *Elevator // Recieved state - others
	StateTX chan *Elevator // Transmitted state - self
}

func InitElev(ID string) Elevator {
	requests := make([][]int, NumFloors)

	for i := range requests {
		requests[i] = make([]int, NumButtons)
	}

	for floor := elevio.GetFloor(); floor == -1; floor = elevio.GetFloor(){
		elevio.SetMotorDirection(elevio.MD_Down)
	}
	elevio.SetMotorDirection(elevio.MD_Stop)

	return Elevator{
		ID: ID,
		Floor: elevio.GetFloor(),
		CurrDirn: elevio.MD_Stop,
		Requests: requests,
		State: IDLE,
		IsOnline: true,
	}
}

func InitID() string{
	idPtr := flag.String("Id","","Id of this elevator")
	flag.Parse()

	if *idPtr != ""{
		return *idPtr
	}

	localIP,err:= localip.LocalIP()
	if err!= nil {
		fmt.Fprintf(os.Stderr, "Warning: Could not retrieve local IP: %v/n", err)
		localIP = "Unknown"

	}
	return fmt.Sprintf("%s-%d", localIP, os.Getpid())
}

type HallRequestAssignment struct {
	ID string 
	UpRequests []bool
	DownRequests []bool
}
type AssignmentResults struct{
	Assignments []HallRequestAssignment
}
