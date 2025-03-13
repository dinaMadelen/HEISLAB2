package main

import (
	"Driver-go/config"
	"Driver-go/elevator"
	"Driver-go/elevio"
	"flag"
	"fmt"
	"strconv"
)

var Port int
var id int

func main() {

	port := flag.Int("port", 15657, "Elevator server port (default: 15657)") // The flag package shows available command-line options with "go run main.go -help".
	elevatorId := flag.Int("id", 0, "Elevator ID (default: 0)")
	flag.Parse()

	Port = *port
	id = *elevatorId

	elevio.Init("localhost:"+strconv.Itoa(Port), config.NumFloors)

	fmt.Println("Elevator initialized.")
	fmt.Println("Elevator ID:", id, "Conntected to port:", Port)
	fmt.Println("Configuration: Floors =", config.NumFloors, ", Elevators =", config.NumElevators)

	buttonEventC := make(chan elevio.ButtonEvent)
	newOrderC := make(chan elevator.Orders)
	deliveredOrderC := make(chan elevio.ButtonEvent)
	newStateC := make(chan elevator.State)

	// 🔹 Start nødvendige goroutines
	go elevio.PollButtons(buttonEventC)
	go elevator.Elevator(newOrderC, deliveredOrderC, newStateC)

	for {
		select {
		case btn := <-buttonEventC:
			fmt.Println("Button pressed:", btn)
			var order elevator.Orders
			order[btn.Floor][btn.Button] = true
			newOrderC <- order

		case delivered := <-deliveredOrderC:
			fmt.Println("Order delivered:", delivered)

		case state := <-newStateC:
			fmt.Println("\nNew state:", state)
		}
	}
}
