package main

import (
	"Primary/elevator"
	"Primary/elevator/config"
	"Primary/elevator/elevio"
	"Primary/master"
	"Primary/network"
)

func main() {
	for i := 0; i < len(master.Ports); i++ {
		master.OptimalChannels[i] = make(chan elevio.OptimalButtonEvent)
		go elevator.Elevator_init(master.Ports[i], config.NumFloors, i+1, master.OptimalChannels[i])
	}

	go master.HallRequest_assigner()
	go network.SendHeartBeat("4000", "localhost")

	select {}

}
