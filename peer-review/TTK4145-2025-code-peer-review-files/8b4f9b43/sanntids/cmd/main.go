package main

import (
	"fmt"
	"time"
	"flag"
	"Driver-go/elevio"
	"sanntids/cmd/localOrders"
	"sanntids/cmd/networkOrders"
	"sanntids/cmd/network/broadcastState"
	"sanntids/cmd/localElevator/structs"
	"sanntids/cmd/localElevator/config"
	"sanntids/cmd/localElevator/fsm"
	"Network-go/network/localip"
)

func main() {
	// Parse command-line arguments
	port := flag.String("port", "15657", "Port number for elevator simulator")
	elevatorID := flag.String("id", "", "Elevator ID (defaults to local IP if not specified)")
	broadcastPortFlag := flag.Int("broadcast", 30003, "Port for broadcasting state")
	numFloorsFlag := flag.Int("floors", config.N_FLOORS, "Number of floors")
	flag.Parse()

	// Configure the simulator
	numFloors := *numFloorsFlag
	elevPort := fmt.Sprintf("localhost:%s", *port)

	// if not set use ip adress
	if *elevatorID == "" {
		*elevatorID, _ =localip.LocalIP()
	}
	fmt.Printf("Local elevator ID: %s, Network port: %d\n", *elevatorID, *broadcastPortFlag)

	// Initialize the elevator driver
	elevio.Init(elevPort, numFloors)

	// Create channels for driver inputs
	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	// Start polling inputs concurrently
	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)
	
	// FSM and state channels
	stateRequestTx := make(chan chan fsm.ElevatorState)
	stateChannel := make(chan fsm.ElevatorState)
	
	// Local order channels
	outgoingOrdersChan := make(chan structs.HallOrder)
	outgoingElevStateChan := make(chan structs.HRAElevState)
	
	// Network communication channels
	incomingNetworkData := make(chan structs.ElevatorDataWithID)
	outgoingNetworkData := make(chan structs.ElevatorDataWithID)
	
	// Nil will be swapped out with order later
	go fsm.Fsm(nil, drv_floors, drv_obstr, drv_stop, stateRequestTx)
	
	// Start a goroutine to periodically request and forward state
	go func() {
		for {
			stateChan := make(chan fsm.ElevatorState)
			stateRequestTx <- stateChan
			state := <-stateChan
			stateChannel <- state
			time.Sleep(time.Second)
		}
	}()

	go localOrders.HallOrderManager(
		drv_buttons,
		stateChannel,
		outgoingOrdersChan,
		outgoingElevStateChan,
	)
	
	go networkOrders.NetworkOrderManager(
		*elevatorID,
		outgoingElevStateChan,
		outgoingOrdersChan,
		incomingNetworkData,
		outgoingNetworkData,
	)
	
	go broadcastState.BroadcastState(outgoingNetworkData, *broadcastPortFlag)
	go broadcastState.ReceiveState(incomingNetworkData, *broadcastPortFlag)

	select {}
}