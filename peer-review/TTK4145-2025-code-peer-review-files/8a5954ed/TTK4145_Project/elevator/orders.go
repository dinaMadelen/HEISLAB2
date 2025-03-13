package elevator

import (
	"datatype"
	"elevio"
	"network"
)

func (e *Elevator) Add_order(order datatype.Queue_element) {
	e.Queue.Add(order)
	elevio.SetButtonLamp(order.Button, order.Floor, true)
	e.Queue.Print()
}

func (e *Elevator) Remove_order(order_floor int) {
	removed := e.Queue.Remove(order_floor)
	Reset_lamps_at_floor(order_floor)

	e.Choose_target_floor()
	for _, order := range removed {
		if ((order.Button == elevio.BT_HallUp && e.Current_floor > e.Target_floor) || (order.Button == elevio.BT_HallDown && e.Current_floor < e.Target_floor)) && len(removed) > 1 {
			e.Add_order(order)
		} else {
			e.Handle_removed_orders(order)
		}
	}
	e.Queue.Print()
}

// If connected, send message to master else add to finished queue for later sending
func (e *Elevator) Handle_removed_orders(order datatype.Queue_element) {
	if e.Connected {
		network.Send_message(e.Conn,
							 datatype.HeaderType.OrderFulfilled,
							 datatype.DataPayload{OrderFloor:  order.Floor,
												  OrderButton: order.Button,
												  ID:          e.ID})
	} else {
		e.Finished_queue.Add(order)
	}
}

func (e *Elevator) Remove_non_cab_orders() {
	temp := make([]datatype.Queue_element, 0, e.Queue.Capacity)
	for _, order := range e.Queue.Orders {
		if order.Button == elevio.BT_Cab {
			temp = append(temp, order)
		} else {
			elevio.SetButtonLamp(order.Button, order.Floor, false)
		}
	}
	e.Queue.Orders = temp
	e.Queue.Print()
}

// Sending confirmation of orders fulfilled while elevator was disconnected
func (e *Elevator) Resend_finished_orders() {
	if e.Finished_queue.Length() > 0 {
		for _, order := range e.Finished_queue.Orders {
			network.Send_message(e.Conn,
								 datatype.HeaderType.OrderFulfilled,
								 datatype.DataPayload{OrderFloor:  order.Floor,
													  OrderButton: order.Button,
													  ID:          e.ID})
			e.Finished_queue.Remove(order.Floor)
		}
	}
}

func (e *Elevator) Choose_target_floor() {
	if e.Queue.Length() > 0 {
		e.Target_floor = e.Queue.Orders[0].Floor

		if e.Target_floor > e.Current_floor {
			e.Target_floor = e.Queue.Find_lowest_between(e.Current_floor)
		} else {
			e.Target_floor = e.Queue.Find_highest_between(e.Current_floor)
		}
	}
}
