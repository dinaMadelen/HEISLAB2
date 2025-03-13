package localOrders

import (
	"Driver-go/elevio"
	"sanntids/cmd/localElevator/config"
	"sanntids/cmd/localElevator/structs"
	"sanntids/cmd/localElevator/fsm"
	"sanntids/cmd/localElevator/elevator"
)

// HallOrderManager listens for button events.
// For BT_Cab events, it updates the CabRequestList and includes it in the HRAElevState.
// For hall events, it sends back a new HallOrder via outgoingOrdersChan.
func HallOrderManager(
	localRequest <-chan elevio.ButtonEvent,
	elevatorState <-chan fsm.ElevatorState,
	outgoingOrdersChan chan<- structs.HallOrder,
	outgoingElevStateChan chan<- structs.HRAElevState,
) {
	// Initialize the cab request list as an array to false
	var cabRequests structs.CabRequestList
	for i := range cabRequests {
		cabRequests[i] = false
	}

	// Initialize with default values
	currentState := structs.HRAElevState{
		Behavior:    "idle",
		Floor:       0,
		Direction:   "stop",
		CabRequests: cabRequests,
	}

	// Start a goroutine to listen for elevator state updates
	go func() {
		for {
			state := <-elevatorState
			currentState.Behavior = elevator.Eb_toString(state.Behaviour)
			currentState.Floor = state.Floor
			currentState.Direction = elevator.Md_toString(state.MotorDirection)
			outgoingElevStateChan <- currentState
		}
	}()

	for {
		select {
		case request := <-localRequest:
			// Handle cab button press
			if request.Button == elevio.BT_Cab {
				if request.Floor >= 0 && request.Floor < config.N_FLOORS {
					cabRequests[request.Floor] = true
					currentState.CabRequests = cabRequests
					outgoingElevStateChan <- currentState
				}
				continue
			}
			
			// For hall button events, create and send a new HallOrder
			newOrder := structs.HallOrder{
				Status:      structs.New,
				DelegatedID: "undelegated",
				Floor:       request.Floor,
				Dir:         request.Button,
			}
			outgoingOrdersChan <- newOrder
		}
	}
}