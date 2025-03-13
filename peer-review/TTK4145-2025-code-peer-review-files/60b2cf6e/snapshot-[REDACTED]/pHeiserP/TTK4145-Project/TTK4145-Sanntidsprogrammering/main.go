package main

import (
	"TTK4145-Heislab/configuration"
	"TTK4145-Heislab/driver-go/elevio"
	"TTK4145-Heislab/single_elevator"
	"fmt"
)

func main() {
	fmt.Println("Elevator System Starting...")

	numFloors := configuration.NumFloors
	elevio.Init("localhost:15657", numFloors)

	newOrderChannel := make(chan single_elevator.Orders, configuration.Buffer)
	completedOrderChannel := make(chan elevio.ButtonEvent, configuration.Buffer)
	newLocalStateChannel := make(chan single_elevator.State, configuration.Buffer)
	buttonPressedChannel := make(chan elevio.ButtonEvent)

	go elevio.PollButtons(buttonPressedChannel)

	// go single_elevator.OrderManager(newOrderChannel, completedOrderChannel, buttonPressedChannel)
	//go order_manager.Run(newOrderChannel, completedOrderChannel, buttonPressedChannel, network_tx, network_rx) - order manager erstattes
	go single_elevator.SingleElevator(newOrderChannel, completedOrderChannel, newLocalStateChannel)
	select {}
}
