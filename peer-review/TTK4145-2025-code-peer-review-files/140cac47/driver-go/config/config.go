package config

const NumFloors = 4
const NumButtons = 3
const NumElevators = 3
const LocalElevator = 0
const DoorOpenDuration = 3
const StateUpdatePeriodsMs = 500
const ElevatorStuckTolerance = 5
const ReconnectTimer = 3
const NumPeerPort = 45678
const NumBcastPort = 45680


type Direction int

const(
	Up Direction = 1
	Down Direction = -1
	Stop Direction = 0
)

type RequestState int 

const(
	None RequestState = iota
	Order
	Confirmed
	Complete
)

type Behaviour int

const(
	Idle Behaviour= iota
	DoorOpen
	Moving
	Unavailable
)

type ButtonType int

const(
	HallUp ButtonType = iota
	HallDown
	Cab
)

type Requests struct{
	Floor int
	Button ButtonType
}


type ElevatorDistributor struct{
	ID string
	Floor int
	Direction Direction
	Requests [][]RequestState
	Behaviour Behaviour
}

type CostRequest struct{
	ID string
	Cost int
	AssignedID string
	Request Requests
}
