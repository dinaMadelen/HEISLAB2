package master

import (
	"Primary/elevator/elevio"
)

// --- CHANNEL VARIABLES --- //

var Ports []string = []string{"localhost:15657", "10.22.24.153:15658", "10.22.24.153:15659"}
var OptimalChannels = make([]chan elevio.OptimalButtonEvent, len(Ports))

// --------------- CHANNEL FUNCTIONS --------------- //

// --- LOCAL FUNCTIONS --- //

func broadcastOptimalEvent(event elevio.OptimalButtonEvent) {
	for _, ch := range OptimalChannels {
		ch <- event
	}
}
