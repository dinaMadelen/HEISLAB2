package main

import (
	"fmt"
	"os"
	"tests"

	"Constants"
)

func main() {
	var is_testing bool = false
	var id string = ""
	for i, arg := range os.Args {
		if i == 0 {
			continue
		}

		switch arg {
		case Constants.ARGV_TEST:
			is_testing = true
		case Constants.ARGV_LISTENER_ONLY:
			// Should not connect an elevator, and should start a listener node.
			// Will print the shared state to screen, additionally will log the messages sent on the network.
		case Constants.ARGV_BACKUP:
			// Spawned by local_heartbeat, will be listening to and sending a heartbeat to the main program
			// If loss of heartbeat from backup: Kill backup and spawn new
			// If loss of heartbeat from main: Kill main and takeover as new main.
		case Constants.ARGV_ELEVATOR_ID:
			id = os.Args[i+1]
		default:
			fmt.Printf("Unknown Arg: %s", arg)
		}
	}

	if id == "" {
		fmt.Println("Error. No id!")
		return
	}

	if is_testing {
		tests.Test_Creating_Connection(id)
		return
	}

	/*

		- Spawn Elevator
		- Spawn Shared State
		- Spawn Network Node

		elevator <-> shared state <-> network node

		Connect them via go channels.

	*/
}
