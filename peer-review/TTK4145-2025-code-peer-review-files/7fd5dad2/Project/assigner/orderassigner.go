package assigner

import (
	. "Project/dataenums"
	"Project/hwelevio"
	"fmt"
	"strconv"
)

func Assigner(
	newOrderChannel chan<- [NFloors][NButtons]bool,
	payloadFromElevator <-chan FromDriverToAssigner,
	toNetworkChannel chan<- FromAssignerToNetwork,
	fromNetworkChannel <-chan FromNetworkToAssigner,
	fromAsstoLight chan<- [NFloors][NButtons]ButtonState,
	nodeID string,
) {
	var (
		PayloadFromassignerToNetwork = initPayloadToNetwork()
		prevAssignedOrders           [NFloors][NButtons]bool
		drv_buttons                  = make(chan ButtonEvent)
	)
	myID, err := strconv.Atoi(nodeID)
	if err != nil {
		fmt.Printf("Invalid nodeID: %v\n", err)
		return
	}
	payload := <-payloadFromElevator
	PayloadFromassignerToNetwork = handlePayloadFromElevator(payload,
		PayloadFromassignerToNetwork, nodeID)

	toNetworkChannel <- PayloadFromassignerToNetwork

	go hwelevio.PollButtons(drv_buttons)
	for {
		select {
		case btnEvent := <-drv_buttons:
			PayloadFromassignerToNetwork = buttonPressed(PayloadFromassignerToNetwork,
				nodeID, btnEvent)
			toNetworkChannel <- PayloadFromassignerToNetwork

		case payload := <-payloadFromElevator:
			PayloadFromassignerToNetwork = handlePayloadFromElevator(payload,
				PayloadFromassignerToNetwork, nodeID)

			toNetworkChannel <- PayloadFromassignerToNetwork

		case PayloadFromNetwork := <-fromNetworkChannel:

			PayloadFromassignerToNetwork = handlePayloadFromNetwork(PayloadFromassignerToNetwork,
				PayloadFromNetwork, myID)

			newOrders := assignOrders(PayloadFromNetwork, myID)

			if newOrders != prevAssignedOrders {
				newOrderChannel <- newOrders
				prevAssignedOrders = newOrders
			}
			fromAsstoLight <- updateLightStates(PayloadFromNetwork, myID)

		}
	}

}
