package orderHandler

import (
	"Driver-go/lights"
	. "Driver-go/types"
)


// OrderHandler is a function that handles orders from the master and the local elevator.
// The function takes a struct of channels as input, and runs an infinite loop that listens for incoming messages on the channels.
type orderChannels struct {
	LocalOrderChannel       chan OrderMatrix
	LocalLightsChannel      chan OrderMatrix
	OrdersFromMasterChannel chan GlobalOrderMap
	OrdersToMasterChannel   chan NetworkMessage
	ButtenEventChannel      chan ButtonEvent
	FinishedFloorChannel    chan int
}

func OrderHandler(ch orderChannels, ID string) {
	ordersFromMaster := make(GlobalOrderMap)

	for {
		select {
		case buttonEvent := <-ch.ButtenEventChannel:
			button := []ButtonEvent{buttonEvent}
			orderEvent := OrderEvent{ElevatorID: ID, Completed: false, Orders: button}
			newOrderEvent := NetworkMessage{MsgType: "New OrderEvent", MsgData: orderEvent, Receipient: Master}
			ch.OrdersToMasterChannel <- newOrderEvent

		case ordersFromMaster = <-ch.OrdersFromMasterChannel:
			localRequests := ordersFromMaster[ID]
			ch.LocalOrderChannel <- localRequests
			localLights := localRequests

			for _, requests := range ordersFromMaster {
				localLights = lights.SetHallLights(requests)

			}

			ch.LocalLightsChannel <- localLights

		case floor := <-ch.FinishedFloorChannel:
			orders := []ButtonEvent{}
			for btn := 0; btn < NUMBUTTONTYPE; btn++ {
				button := ButtonEvent{Floor: floor, Button: ButtonType(btn)}
				orders = append(orders, button)

			}

			finishedOrder := OrderEvent{ElevatorID: ID, Completed: true, Orders: orders}
			regFinishedOrder := NetworkMessage{MsgType: "Finished OrderEvent", MsgData: finishedOrder, Receipient: Master}
			ch.OrdersToMasterChannel <- regFinishedOrder

		}
	}
}
