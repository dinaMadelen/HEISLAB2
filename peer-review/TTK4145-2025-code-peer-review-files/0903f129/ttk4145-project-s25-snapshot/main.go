package main

import (
	"Project/elevator"
	"Project/network"
	"Project/network/localip"
	"fmt"
)

func main() {
	// Initialize channels for communication between modules
	localOrderRequest := make(chan elevator.Order)
	addToLocalQueue := make(chan elevator.Order)
	assignOrder := make(chan elevator.OrderUpdate)
	elevid, err := localip.MyId()
	if err != nil {
		fmt.Print("Error: ", err)
	} else {
		go network.PrimaryBackupNetwork(elevid, localOrderRequest, addToLocalQueue, assignOrder) // Start the network module
		go elevator.RunElevatorFSM(elevid, localOrderRequest, addToLocalQueue, assignOrder) 	 // Start the elevator module
	}
	select {} 
}
