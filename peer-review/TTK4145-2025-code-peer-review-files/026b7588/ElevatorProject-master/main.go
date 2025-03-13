package main

import (
	//"ElevatorProject/elevator"
	"ElevatorProject/elevio"
	"ElevatorProject/fsm"
	"ElevatorProject/network"
	"ElevatorProject/timer"
	"flag"
	"time"
)

func main() {

	var port string
	flag.StringVar(&port, "port", "", "port of elevator")
	flag.Parse()

	address := "localhost:" + port

	// Initialize elevator
	numFloors := 4
	prevFloor := -1

	elevio.Init(address, numFloors)
	//elevator.InitElevator(numFloors, elevio.NumButtonTypes)

	// Polling rate configuration
	inputPollRate := 25 * time.Millisecond

	// Event channels for hardware events
	buttonPressCh := make(chan elevio.ButtonEvent)
	floorSensorCh := make(chan int)
	stopButtonCh := make(chan bool)
	obstructionSwitchCh := make(chan bool)
	orderTx := make(chan network.OrderMsg)
	orderRx := make(chan network.OrderMsg)

	// Start polling goroutines
	go elevio.PollButtons(buttonPressCh)
	go elevio.PollFloorSensor(floorSensorCh)
	go elevio.PollStopButton(stopButtonCh)
	go elevio.PollObstructionSwitch(obstructionSwitchCh)
	go network.Network(orderTx, orderRx)

	/*
		var ID string

		go func() {
			ID = network.Network(orderTx, orderRx)
		}()
	*/

	obstructionActive := false
	stop := false

	// Main event loop
	for {
		select {
		case buttonEvent := <-buttonPressCh:
			if stop {
				stop = false
			}
			fsm.FsmOnRequestButtonPress(buttonEvent.Floor, buttonEvent.Button)
			orderTx <- network.OrderMsg{
				Floor:  buttonEvent.Floor,
				Button: buttonEvent.Button,
				Active: true}

		case currentFloor := <-floorSensorCh:
			if currentFloor != prevFloor {
				fsm.FsmOnFloorArrival(currentFloor)
				elevio.SetFloorIndicator(currentFloor)

				if !obstructionActive {
					timer.TimerStop()
					timer.TimerStart(3.0)
				}
			}
			prevFloor = currentFloor
			obstructionActive = false

		case stopPressed := <-stopButtonCh:
			if stopPressed {
				elevio.SetStopLamp(true)
				elevio.SetMotorDirection(0)
				stop = true
			} else {
				elevio.SetStopLamp(false)
			}

		case <-time.After(inputPollRate):
			if timer.TimerTimedOut() {
				fsm.FsmOnDoorTimeout()
				timer.TimerStop()
			}

		case obstruction := <-obstructionSwitchCh:
			if obstruction {
				obstructionActive = true
				timer.TimerStop()
			} else if !obstruction {
				obstructionActive = false
				timer.TimerStop()
				timer.TimerStart(3.0)
			}
		}
	}
}
