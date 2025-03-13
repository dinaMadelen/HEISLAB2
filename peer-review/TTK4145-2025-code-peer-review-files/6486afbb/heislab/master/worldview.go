package master

import (
	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/slave"
)

type Calls struct {
	HallCalls [config.N_FLOORS][config.N_BUTTONS - 1]bool
	CabCalls  [config.N_ELEVATORS][config.N_FLOORS]bool
}

type WorldView struct {
	Elevators      [config.N_ELEVATORS]slave.Elevator
	OwnId          string
	HallCalls      [config.N_FLOORS][config.N_BUTTONS - 1]bool
	CabCalls       [config.N_ELEVATORS][config.N_FLOORS]bool
	AliveElevators [config.N_ELEVATORS]bool
}

type BackupCalls struct {
	Calls Calls
	Id    int
}

type AssignCalls struct {
	Calls          Calls
	AliveElevators [config.N_ELEVATORS]bool
}

type UpdateCalls struct {
	Calls   Calls
	AddCall bool
}
