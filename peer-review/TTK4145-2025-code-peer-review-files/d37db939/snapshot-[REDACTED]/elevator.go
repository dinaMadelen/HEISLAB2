package main

import (
	"time"
)

// elevator states
type ElevState int
const (
	ELEVSTATE_IDLE ElevState = iota
	ELEVSTATE_DOOR_OPEN
	ELEVSTATE_MOVING
)

// how requests should be cleared when stopping at a floor
type ClearRequests int
const (
	CLEAR_ALL ClearRequests = iota
	CLEAR_DIRECTION
)

// the state and configuration of an elevator
type Elevator struct {
	ID       int 						`json:"id"`
	MasterID int
	floor    int						//`json:"floor"`
	dirn     MotorDirection             //`json:"dirn"`
	requests [N_FLOORS][N_BUTTONS]bool  //`json:"requests"`
	state    ElevState
	config   struct {
		clearRequests     ClearRequests
		doorOpenDurationS float64
	}
	active    bool
	lastSeen  time.Time					`json:"-"`
	isMaster  bool      				//`json:"is_master"`
	heartbeat time.Time 				`json:"-"`

 	Elevators    map[int]*Elevator
	stateUpdated bool

	// RYDD
}

// elevator dimensions
const (
	N_FLOORS  = 4
	N_BUTTONS = 3
)

// returns an Elevator instance with default uninitialized values
func elevatorUninitialized() Elevator {
	return Elevator{
		floor: -1,
		dirn:  DIRN_STOP,
		state: ELEVSTATE_IDLE,
		config: struct {
			clearRequests     ClearRequests
			doorOpenDurationS float64
		}{
			clearRequests:     CLEAR_ALL,
			doorOpenDurationS: 3.0,
		},
	}
}