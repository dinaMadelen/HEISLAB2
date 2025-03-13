// common_types/types.go
package commontypes

type ElevState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type HRAInput struct {
	HallRequests [][2]bool            `json:"hallRequests"`
	States       map[string]ElevState `json:"states"`
}

type AllElevatorStates struct {
	NumElevators   int
	ElevatorStates []ElevState
	AliveStates    []bool
}
