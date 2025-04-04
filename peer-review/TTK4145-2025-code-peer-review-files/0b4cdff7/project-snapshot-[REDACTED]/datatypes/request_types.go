package datatypes

import "project/elevio"

type RequestState int

const (
	Completed  RequestState = 0
	Unassignes RequestState = 1
	Assigned   RequestState = 2
)

type RequestType struct {
	State     RequestState
	Count     int
	AwareList []string
}

type ElevatorInfo struct {
	Available bool
	Behaviour ElevBehaviour
	Direction elevio.MotorDirection
	Floor     int
}

type NetworkMsg struct {
	SenderID           string
	Available          bool
	Behavior           ElevBehaviour
	Direction          elevio.MotorDirection
	Floor              int
	SenderHallRequests [N_FLOORS][N_BUTTONS]RequestType
	allCabRequests     map[string][N_FLOORS]RequestType
}
