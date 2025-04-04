package main

import (
	"flag"
	"fmt"
	"project/datatypes"
	"project/elevator_control"
	"project/elevio"
	"project/network/bcast"
	"project/network/peers"
	"time"
)

func main() {

	idFlag := flag.String("id", "ElevDefault", "Unique ID for this elevator")
	portFlag := flag.String("port", "15657", "Simulator port") // Define both flags first

	flag.Parse() // Call flag.Parse() only once

	myID := *idFlag
	port := *portFlag

	numFloors := 4
	elevio.Init("localhost:"+port, numFloors)

	//channels for peers
	txPeerEnable := make(chan bool)
	rxPeerUpdates := make(chan peers.PeerUpdate)

	// start the transmitter/receiver for peers
	go bcast.Transmitter(17658, txPeerEnable)
	go bcast.Receiver(17658, rxPeerUpdates)

	//Start broadcasting ID
	txPeerEnable <- true

	//channels for broadcasting elevator state
	txElevatorState := make(chan datatypes.NetElevator)
	rxElevatorState := make(chan datatypes.NetElevator)

	// start the transmitter/receiver for states
	go bcast.Transmitter(17657, txElevatorState)
	go bcast.Receiver(17657, rxElevatorState)

	elevator := elevator_control.InitializeFSM()
	knownElevators := make(map[string]datatypes.NetElevator)
	context := elevator_control.GetElevatorContext(myID)

	//Heratbeat broadcasting state,
	go func() {

		for {
			currentState := datatypes.NetElevator{
				ID:           myID,
				CurrentFloor: elevator.CurrentFloor,
				Direction:    elevator.Direction,
				State:        elevator.State,
				Orders:       elevator.Orders,
				StopActive:   elevator.StopActive,
			}
			txElevatorState <- currentState
			time.Sleep(100 * time.Millisecond)
		}
	}()

	//Listen for states
	go func() {
		for {
			tempState := <-rxElevatorState
			knownElevators[tempState.ID] = tempState
			fmt.Println("Received state from:", tempState.ID, "Floor:", tempState.CurrentFloor)
		}
	}()

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	fmt.Println("Elevator system ready. MyID =", myID)

	for {
		select {
		case a := <-drv_buttons: // a tilsvarer knappetrykket
			// håndtere trykk på knapper
			elevator_control.OnRequestButtonPress(&elevator, a.Floor, a.Button, context)

		case a := <-drv_floors: // a blir etasjen man ankommer
			elevator_control.OnFloorArrival(&elevator, a, context)

		case a := <-drv_obstr: // håndterer dersom obstruction blir aktivert
			if a {

				elevio.SetMotorDirection(elevio.MD_Stop)
			} else {
				elevio.SetMotorDirection(elevator.Direction)
			}

		case <-drv_stop: // håndterer dersom stop blir trykket
			elevator_control.OnStopButtonPress(&elevator)

		}

		elevator_control.UpdateLights(&elevator)
	}
}
