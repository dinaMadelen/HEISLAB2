package main

import (
	"flag"
	"fmt"

	"HeisLab2025/HRA"
	"HeisLab2025/elev_algo"
	elev "HeisLab2025/elev_algo/elevator_io"
	"HeisLab2025/nettverk"
)

//overview:
//the project is currently divided into 3 modules: single elevator ("elev_algo"), networking ("nettverk") and Hall request assigner ("HRA")
// elev_algo: Handles the sigle elevator (requests, input, motor, etc.)
// nettverk: Handles messaging over the network, and some convertion 
// HRA: Feeds a map of elevators to the executable and returns the relevant output to the elev_algo module


func main() {
	fmt.Println("Starting, please wait...")

	var (
		id      string
		simPort string
	)

	flag.StringVar(&id, "id", "0", "id of this peer")
	flag.StringVar(&simPort, "simPort", "15678", "simulation server port")
	flag.Parse()

	SingElevChans := elev_algo.SingleElevatorChans{
		Drv_buttons:       make(chan elev.ButtonEvent),
		Drv_floors:        make(chan int),
		Drv_obstr:         make(chan bool),
		Drv_stop:          make(chan bool),
		Timer_channel:     make(chan bool),
		Single_elev_queue: make(chan [][2]bool),
	}

	ch_HRAOut := make(chan map[string][][2]bool)
	ch_HRAInputTx := make(chan nettverk.InformationElev)
	ch_HRAInputRx := make(chan nettverk.InformationElev)

	go elev_algo.Elev_main(SingElevChans, simPort)
	go nettverk.Nettverk_hoved(ch_HRAInputRx, id)
	go HRA.HRAMain(ch_HRAOut)
	go nettverk.SetElevatorStatus(ch_HRAInputTx)
	go nettverk.RecieveElevatorStatus(ch_HRAInputRx)
	go nettverk.BroadcastElevatorStatus(ch_HRAInputTx)
	go nettverk.FromHRA(ch_HRAOut, SingElevChans.Single_elev_queue)

	select {}

}

