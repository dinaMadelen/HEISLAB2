package structs

import (
	"Driver-go/elevio"
	"sanntids/cmd/localElevator/config"
)

type OrderStatus int

const (
	Unknown OrderStatus = iota - 1
	New
	Assigned
	Confirmed
	Completed
)

// HallOrder structure for order management
type HallOrder struct {
	DelegatedID string
	Status      OrderStatus
	Floor       int
	Dir         elevio.ButtonType
}

type CabRequestList [config.N_FLOORS]bool

type HRAElevState struct {
	Behavior    string 
	Floor       int
	Direction   string
	CabRequests CabRequestList
}

type ElevatorDataWithID struct {
	ElevatorID string
	ElevatorState map[string]HRAElevState
	HallOrders    []HallOrder
}