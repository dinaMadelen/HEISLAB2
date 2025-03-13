package main

// The main node is used to run elevator and controller concurrently
// The main node is also used to run the network module

import (
	"flag"
	"fmt"
	"main/elev_algo_go/elevator"
	"main/elevio"
)


// Global variables
var (
	elev_to_ctrl_button_chan          = make(chan elevio.ButtonEvent)
	elev_to_ctrl_chan                 = make(chan elevator.Elevator)
	ctrl_to_elev_chan          = make(chan elevio.ButtonEvent)
	obstruction_timer_duration int
)

func main() {
	debug := flag.Bool("debug", false, "Enable debug mode") // Command line flag to enable debug mode
	flag.Parse()

	if *debug { 							// If debug mode is enabled, set obstruction timer duration to 5 seconds instead of 30
		fmt.Println("Debug mode enabled")
		obstruction_timer_duration = 5
	} else {
		fmt.Println("Running in normal mode")
		obstruction_timer_duration = 30
	}

	fmt.Println("Starting controller and elevetor")

	go main_controller() // Start the controller
	go main_elevator() // Start the elevator
	for { // Infinite loop to keep the main node running
	}

}
