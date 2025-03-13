package master

import (
	"datatype"
	"elevio"
	"fmt"
)

type Master_queue struct {
	Orders   []datatype.Master_queue_element
	Capacity int
}

func Create_master_queue(capacity int) *Master_queue {
	return &Master_queue{
		Orders:   make([]datatype.Master_queue_element, 0, capacity),
		Capacity: capacity,
	}
}

func (q *Master_queue) Length() int {
	return len(q.Orders)
}

// Check if order already exists in the queue
func (q *Master_queue) Check_for_duplicate(order datatype.Master_queue_element) bool {
	exists := false
	for _, ord := range q.Orders {
		if (ord.Button == order.Button && ord.Floor == order.Floor) && ord.ReceivedFrom == order.ReceivedFrom {
			exists = true
		}
	}
	return exists
}

func (q *Master_queue) Add(order datatype.Master_queue_element) {
	if q.Length() < q.Capacity {
		q.Orders = append(q.Orders, order)
	}
}

func (q *Master_queue) Remove(floor int, button elevio.ButtonType, sentTo string) {
	temp := make([]datatype.Master_queue_element, 0, q.Capacity)
	for _, order := range q.Orders {
		if !(order.Floor == floor && order.Button == button && order.SentTo == sentTo) {
			temp = append(temp, order)
		}
	}
	q.Orders = temp
}

func (q *Master_queue) Update_send_to(floor int, button elevio.ButtonType, fromID string, toID string) {
	for i := range q.Orders {
		if q.Orders[i].Floor == floor && q.Orders[i].Button == button && q.Orders[i].ReceivedFrom == fromID {
			q.Orders[i].SentTo = toID
		}
	}
}

func (q *Master_queue) Print() {
	fmt.Printf("Queue: \n")
	for i, order := range q.Orders {
		switch order.Button {
		case elevio.BT_HallUp:
			fmt.Printf("Order %d: floor %d, hall up button, received from elevator (%s), send to elevator (%s). \n", i, order.Floor, order.ReceivedFrom, order.SentTo)
		case elevio.BT_HallDown:
			fmt.Printf("Order %d: floor %d, hall down button, received from elevator (%s), send to elevator (%s). \n", i, order.Floor, order.ReceivedFrom, order.SentTo)
		case elevio.BT_Cab:
			fmt.Printf("Order %d: floor %d, cab button, received from elevator (%s), send to elevator (%s). \n", i, order.Floor, order.ReceivedFrom, order.SentTo)
		}
	}
	fmt.Println()
}
