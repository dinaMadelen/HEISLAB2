package main

import (
	"Driver-go/elevator/driver"
	"Driver-go/elevator/types"
	request_control "Driver-go/requests"
	"os"
)

func main() {
	// Initialize the driver connection to the elevator server
	addr := os.Args[1]
	addr = "localhost:" + addr
	driver.Init(addr, types.N_FLOORS)

	requestsCh := make(chan [types.N_FLOORS][types.N_BUTTONS]bool)
	completedCh := make(chan types.ButtonEvent)

	types.InitElevator()
	go request_control.RunRequestControl(
		"elevator1",
		requestsCh,
		completedCh,
	)
}
