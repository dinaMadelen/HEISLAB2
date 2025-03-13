package types

import (
	"heisV5/config"
)

// HallRequestAssignerState represents an elevator's state as required by the hall request assigner.
type HallRequestAssignerState struct {
	Behaviour   string                 `json:"behaviour"`
	Floor       int                    `json:"floor"`
	Direction   string                 `json:"direction"`
	CabRequests [config.NumFloors]bool `json:"cabRequests"`
}

// HallRequestAssignerDesiredInput represents the full input structure for the hall request assigner.
type HallRequestAssignerDesiredInput struct {
	HallRequests [config.NumFloors][2]bool           `json:"hallRequests"`
	States       map[string]HallRequestAssignerState `json:"states"`
}

type Orders [config.NumFloors][config.NumButtons]bool
