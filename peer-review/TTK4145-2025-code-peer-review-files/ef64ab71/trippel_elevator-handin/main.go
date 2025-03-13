package main

import (
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/delegateOrders"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
	"github.com/Eirik-a-Johansen/trippel_elevator/fsm"
	"github.com/Eirik-a-Johansen/trippel_elevator/initialize"
	"github.com/Eirik-a-Johansen/trippel_elevator/input"
	"github.com/Eirik-a-Johansen/trippel_elevator/mergeOrders"
	"github.com/Eirik-a-Johansen/trippel_elevator/network"
)

/*
starts the program

Contains most go-routines:

	Network.go: starts 5 go-routines for sending and reciving messages
	input.go: starts 3 go-routines for reciving all 3 different button-types
*/
func main() {
	initialize.Init(&elevator.LocalElevator)

	go input.Recive_buttons(&elevator.LocalElevator)
	go fsm.Fsm(&elevator.LocalElevator)
	go mergeOrders.MergeOrders(&elevator.LocalElevator)
	go delegateOrders.DelegateOrders(&elevator.LocalElevator)
	go network.Network(&elevator.LocalElevator)

	for {
		time.Sleep(300 * time.Millisecond)
	}
}
