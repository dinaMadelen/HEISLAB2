package elevatordriver

import (
	// for printing
	"log"
	"testing"

	// Assuming PollRequests is in this package
	"group48.ttk4145.ntnu/elevators/elevatorio"
	"group48.ttk4145.ntnu/elevators/models" // Assuming your models are here
)

func TestStarter(t *testing.T) {
	pollObstructionSwitch := make(chan bool)
	pollFloorSensor := make(chan int)
	pollOrders := make(chan models.Orders)
	resolvedRequests := make(chan models.RequestMessage)

	receiver := make([]chan<- models.ElevatorState, 0)
	id := models.Id(3)

	//For the test:
	receiverRequest := make(chan models.RequestMessage)

	go testPollOrders(pollOrders, receiverRequest)
	go testPollResolvedRequest(resolvedRequests)
	go elevatorio.PollRequests(receiverRequest)
	go elevatorio.PollFloorSensor(pollFloorSensor)
	go elevatorio.PollObstructionSwitch(pollObstructionSwitch)

	Starter(pollObstructionSwitch, pollFloorSensor, pollOrders, resolvedRequests, receiver, id)

}

func testPollOrders(receiver chan<- models.Orders, receiverRequest chan models.RequestMessage) {
	orders := initOrders(NFloors)
	for order_request := range receiverRequest {
		orders[order_request.Request.Origin.Floor][order_request.Request.Origin.ButtonType] = true
		receiver <- orders
		log.Printf("Orders from testPollOrders: %v", orders)
	}
}

func testPollResolvedRequest(receiver <-chan models.RequestMessage) {
	for {
		log.Printf("ResolvedRequest: %v", <-receiver)
	}
}
