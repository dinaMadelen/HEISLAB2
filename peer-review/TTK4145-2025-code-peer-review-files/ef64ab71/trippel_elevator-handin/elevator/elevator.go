package elevator

/*
This module defines the datastructures and global variables used in the code

ElevatorBehaviour: contains the different states for the state machine. This is for the local elevator
Dirn: direction of travel
orderStructure: For every order there is one value refering to the state of the order, and one list containing all the elevators that agree on this worldview
Elevator: This "is" an elevator, with all information needed

Global variables:
LocalElevator: the elevator this pc runs
Elevators: List of all the elevators in the system. Used for collecting all information about the system
Delegated: A list with an overview about wich elevator is delegated an order
*/

import (
	"sync"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
)

type ElevatorBehaviour int

const (
	EB_Idle     ElevatorBehaviour = 0
	EB_DoorOpen ElevatorBehaviour = 1
	EB_Moving   ElevatorBehaviour = 2
	EB_Stop     ElevatorBehaviour = 3
)

type Dirn int

const (
	D_Stop Dirn = 0
	D_Up   Dirn = 1
	D_Down Dirn = 2
)

type orderStructure struct {
	Value int
	List  []int
}

type Elevator struct {
	Floor           int
	Dirn            Dirn
	Orders          [driver.N_Floors][driver.N_Buttons + (NumberOfElevators - 1)]orderStructure
	Behaviour       ElevatorBehaviour
	OnFloor         bool
	OpenDoor        bool
	DoorObstruction bool
	Stop            bool
	DoorTimer       float64

	ID              int
	IsMaster        bool
	MyOrders        [driver.N_Floors][driver.N_Buttons]int
	OnlineElevators [NumberOfElevators]bool
	Functional      bool
}

var (
	LocalElevator Elevator
	Mutex         sync.Mutex
	Elevators     [NumberOfElevators]Elevator
	Delegated     [driver.N_Floors][driver.N_Buttons + (NumberOfElevators - 1)]int
)

const NumberOfElevators = 3
