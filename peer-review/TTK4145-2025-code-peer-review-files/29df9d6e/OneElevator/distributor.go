package main

import (
	"fmt"
)

type AckStatus int

const (
	NotAcked AckStatus = iota
	Acked
	Unavailable
)

type Order struct {
	id int
	masterID int
	currState int
	floor int
	button int
}

type Distributor struct {
	id int // ID of the elevator
	elevator Elevator
	orders map[int]Order // Map of all orders
	//elevatorsOnline map[int]bool // Map of all elevators online
	Ackmap [N_FLOORS]AckStatus
}

// trenger en gorroutine i main som sammenligner sin egen counter med alle 
// andre, inkrementerer når alle er lik eller større.
// Må lage en distributor for kommunikasjon. Structen til distributor må inneholde hash
// map med alle kjente ordrer, hvilke heiser som lever og ID for hesen den er distrubutor for. 


func NewOrder(elevatorID int, counter int, floor int, button int) Order {
    return Order{
        id:      elevatorID*1000 + counter, // Unique order ID
        masterID:  elevatorID,              // This elevator received the hall call
        currState: 1,      
		floor: floor, 						// Floor where the order was made
		button: button,                    // Initial counter value
    }
}

func newDistibutor(id int, elevator Elevator) Distributor {
	return Distributor{
		id: id,
		elevator: elevator,
		orders: make(map[int]Order),
		elevatorsOnline: make(map[int]bool),
	}
}

func (dis *Distributor) deleteOrder(order.id){
	delete(dis.orders, order.id)
}

func (dis *Distributor) isAcknowledged(id int) bool {
	for i := range dis.Ackmap {
		if dis.Ackmap[i] != Acked {
			return false
		}
	}
	return true 
}

for {
	// Check if any orders are made
	// Denne skal settes inn der hvor one elevator mottar en ordre!
	select {
	case request := <-ButtonPressCh:
		// Add order to map
		order = NewOrder(Distributor.id, 1, request.Floor, request.Button)
		Distributor.orders[order.id] = order
	}
}

