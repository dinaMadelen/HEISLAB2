package msgTypes

import (
	"Heis/pkg/elevator"
)


type UdpMsg struct {
	Message        string
	Iter           int
	ElevatorStateMsg *ElevatorStateMsg // Pointer to ButtonPressMsg, nil if not a button press
}


type ElevatorStateMsg struct {
	Message        string
	Iter           int
	Elevator elevator.Elevator
	Id       string
}

