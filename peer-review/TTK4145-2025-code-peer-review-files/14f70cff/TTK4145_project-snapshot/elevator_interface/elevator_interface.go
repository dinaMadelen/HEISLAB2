package elevator_interface

import (
	"elevator/elevio"
	"reflect"
	"time"
)

type OrderStatus int

const (
	NoOrder OrderStatus = iota
	OrderUnconfirmed
	OrderConfirmed
	OrderUnknown
)

type Worldview struct {
	Elevators map[string]elevio.Elevator
	Orders    map[string][elevio.N_FLOORS][elevio.N_BUTTONS]OrderStatus
}

var currentWorldview Worldview
var myID string

func updateOrderStatus(currentOrderStatus OrderStatus, incomingOrderStatus OrderStatus) OrderStatus {
	newOrderStatus := currentOrderStatus
	if currentOrderStatus == OrderUnknown {
		newOrderStatus = incomingOrderStatus
	} else if currentOrderStatus == NoOrder && incomingOrderStatus == OrderUnconfirmed {
		newOrderStatus = OrderUnconfirmed
	} else if currentOrderStatus == OrderUnconfirmed && incomingOrderStatus == OrderConfirmed {
		newOrderStatus = OrderConfirmed
	} else if currentOrderStatus == OrderConfirmed && incomingOrderStatus == NoOrder {
		newOrderStatus = NoOrder
	}

	return newOrderStatus
}

func updateWorldview(incomingWorldview Worldview) {
	for id, elevator := range incomingWorldview.Elevators {
		if id != myID {
			currentWorldview.Elevators[id] = elevator
		}
	}

	for id, incomingOrder := range incomingWorldview.Orders {
		currentOrder := currentWorldview.Orders[id]
		if !reflect.DeepEqual(incomingOrder, currentOrder) {

			var newOrder [elevio.N_FLOORS][elevio.N_BUTTONS]OrderStatus
			for f := 0; f < elevio.N_FLOORS; f++ {
				newOrder[f][elevio.BT_HallUp] = updateOrderStatus(currentOrder[f][elevio.BT_HallUp], incomingOrder[f][elevio.BT_HallUp])
				newOrder[f][elevio.BT_HallDown] = updateOrderStatus(currentOrder[f][elevio.BT_HallDown], incomingOrder[f][elevio.BT_HallDown])
				newOrder[f][elevio.BT_Cab] = updateOrderStatus(currentOrder[f][elevio.BT_Cab], incomingOrder[f][elevio.BT_Cab])
			}

			currentWorldview.Orders[id] = newOrder
		}
	}

}

func SendWorldview(ch chan Worldview) {
	for {
		ch <- currentWorldview
		time.Sleep(1 * time.Second)
	}
}
