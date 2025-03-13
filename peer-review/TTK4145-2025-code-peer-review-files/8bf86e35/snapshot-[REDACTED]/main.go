package main

import (
	"Elevator/elevatorControl"
	"flag"
	//"fmt"
)

func main() {
	elevatorIDPtr := flag.Int("id", 0, "ID of the elevator")
	//flag.Parse()

	elevatorID := *elevatorIDPtr

	portNumberPtr := flag.Int("port", 15657, "Port Number of the elevator")
	flag.Parse()

	portNumber := *portNumberPtr

	go elevatorControl.StartManager(elevatorID, portNumber)

	select {}

}
