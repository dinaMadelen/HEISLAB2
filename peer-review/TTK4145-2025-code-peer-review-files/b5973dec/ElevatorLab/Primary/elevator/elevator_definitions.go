package elevator

import (
	"Primary/elevator/config"
	"Primary/elevator/elevio"
)

// --- VARIABLES --- //

var state State
var Hall = make(chan elevio.ButtonEvent)
var Elevators []Elevator = []Elevator{elevator_initialize(), elevator_initialize(), elevator_initialize()}

// --- STRUCT DEFINITIONS --- //

type Elevator struct {
	Floor        int
	Dirn         elevio.MotorDirection
	Request      [config.NumFloors][config.NumButtons]bool
	Behaviour    config.ElevatorBehaviour
	Config       config.Config
	TimerEndTime float64
	TimerActive  int
}

type State struct {
	Elevator_id        int
	Elevator_floor     int
	Elevator_dir       int
	Elevator_behaviour int
	Elevator_request   [config.NumFloors][config.NumButtons]bool
}

type DirnBehaviourPair struct {
	dirn      elevio.MotorDirection
	behaviour config.ElevatorBehaviour
}
