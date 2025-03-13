package config

import (
	"time"
)

const (
	IDLE ElevatorState = iota
	MOVING
	DOOR_OPEN
)

const (
	NUM_FLOORS    = 4
	NUM_BUTTONS   = 3
	NUM_ELEVATORS = 1 // FOR NOW
)

const (
	T_HEARTBEAT = time.Millisecond*50 //Must be much faster than .5 s
	T_SLEEP = time.Millisecond*20
	T_DOOR_OPEN = time.Second*3
	T_OBSTRUCTED_PRIMARY = time.Second*3
	T_OBSTRUCTED_LOCAL = time.Second*4
	T_TRAVEL = time.Second*2 	//Approximate time to travel from floor i to floor i+-1
	T_PRIMARY_TIMEOUT = time.Millisecond*500
	T_BLINK = time.Millisecond*100
)

const (
	UP   = 1
	DOWN = -1
	STOP = 0
)

const(
	Obstructed = iota
	Disconnected
)

// TODO: Only two ports necessary
const (
	PORT_PEERS      = 20020
	PORT_ELEVSTATE  = 20030
	PORT_WORLDVIEW  = 20040
	PORT_REQUEST    = 20050
	PORT_ORDER      = 20060
	PORT_HALLLIGHTS = 20070
)

type ElevatorState int


type Elevator struct {
	Id            string
	Floor         int
	Direction     int
	PrevDirection int
	State         ElevatorState
	Orders        [NUM_FLOORS][NUM_BUTTONS]bool
  Obstructed bool
}

type Order struct {
	Id     string
	Floor  int
	Button int
}

func OrderConstructor(Id string, Floor int, Button int) Order {
	return Order{Id: Id, Floor: Floor, Button: Button}
}
type PeerUpdate struct {
	Peers []string
	New   string
	Lost  []string
}

//----------------PRIMARY/BACKUP--------------------

type Worldview struct {
	PrimaryId     string
	PeerInfo      PeerUpdate
	FleetSnapshot map[string]Elevator // Owned by
}

func WorldviewConstructor(PrimaryId string, PeerInfo PeerUpdate, FleetSnapshot map[string]Elevator) Worldview {
	return Worldview{PrimaryId: PrimaryId, PeerInfo: PeerInfo, FleetSnapshot: FleetSnapshot}
}

type FleetAccess struct {
	Cmd     string //{"read","write one","write all"}
	Id      string
	Elev    Elevator
	ElevMap map[string]Elevator
	ReadCh  chan map[string]Elevator
}

type Reassignment struct {
	Cause int
	ObsId string //Only relevant for obstructed elevators
}

//--------------------------------------------

