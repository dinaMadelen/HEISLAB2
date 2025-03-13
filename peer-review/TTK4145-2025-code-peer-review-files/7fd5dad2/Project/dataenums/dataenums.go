package dataenums

import "time"

const (
	NFloors           int    = 4
	NButtons          int    = 3
	NElevators        int    = 3
	PollRateMS               = 20 * time.Millisecond
	DoorOpenDurationS        = 3 * time.Second
	MotorTimeoutS            = 4 * time.Second 
	BufferSize               = 4 * 1024
	HeartbeatInterval        = 150 * time.Millisecond  
	HeartbeatTimeout         = 3000 * time.Millisecond 
	MessagePort       int    = 1338
	Addr              string = "localhost:15657"
)

type Button int

const (
	BHallUp Button = iota
	BHallDown
	BCab
)

type ButtonState int

const (
	Initial ButtonState = iota
	Idle
	ButtonPressed
	OrderAssigned
	OrderComplete
)

type HWMotorDirection int

const (
	MDDown HWMotorDirection = iota - 1
	MDStop
	MDUp
)

type ButtonEvent struct {
	Floor  int
	Button Button
}

type ElevatorBehaviour int

const (
	EBIdle ElevatorBehaviour = iota
	EBDoorOpen
	EBMoving
)

type DirnBehaviourPair struct {
	Dirn      HWMotorDirection
	Behaviour ElevatorBehaviour
}

type Elevator struct {
	CurrentFloor     int
	Dirn             HWMotorDirection
	Requests         [NFloors][NButtons]bool
	CurrentBehaviour ElevatorBehaviour
	ActiveSatus      bool
}

type HRAElevState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type HRAInput struct {
	HallRequests [NFloors][2]bool        `json:"hallRequests"`
	States       map[string]HRAElevState `json:"states"`
}

type Message struct {
	//TODO: Make int
	SenderId      string
	ElevatorList  [NElevators]HRAElevState
	HallOrderList [NElevators][NFloors][NButtons]ButtonState
	OnlineStatus  bool
	AliveList     [NElevators]bool
}

type FromAssignerToNetwork struct {
	HallRequests [NFloors][NButtons]ButtonState
	States       map[string]HRAElevState
	ActiveSatus  bool
}

type FromNetworkToAssigner struct {
	AliveList     [NElevators]bool
	ElevatorList  [NElevators]HRAElevState
	HallOrderList [NElevators][NFloors][NButtons]ButtonState
}

type FromDriverToLight struct {
	CurrentFloor int
	DoorLight    bool
}

type FromDriverToAssigner struct {
	Elevator        Elevator
	CompletedOrders [NFloors][NButtons]bool
}

type NetworkNodeRegistry struct {
	Nodes []string
	New   []string
	Lost  []string
}
