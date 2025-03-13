package elevator

import (
	"datatype"
	"elevio"
	"fmt"
)

type Queue struct {
	Orders   []datatype.Queue_element
	Capacity int
}

func Create_queue(capacity int) *Queue {
	return &Queue{
		Orders:   make([]datatype.Queue_element, 0, capacity),
		Capacity: capacity,
	}
}

func (q *Queue) Length() int {
	return len(q.Orders)
}

func (q *Queue) Add(order datatype.Queue_element) {
	//Check for duplicates
	exists := false
	for _, ord := range q.Orders {
		if ord == order {
			exists = true
		}
	}

	if q.Length() < q.Capacity && !exists {
		q.Orders = append(q.Orders, order)
	}
}

func (q *Queue) Remove(floor int) []datatype.Queue_element {
	temp := make([]datatype.Queue_element, 0, q.Capacity)
	removed := make([]datatype.Queue_element, 0, q.Capacity)
	for _, order := range q.Orders {
		if order.Floor == floor {
			removed = append(removed, order)
		} else {
			temp = append(temp, order)
		}
	}
	q.Orders = temp
	return removed
}

func (q *Queue) Print() {
	fmt.Printf("Queue: \n")
	for i, order := range q.Orders {
		switch order.Button {
		case elevio.BT_HallUp:
			fmt.Printf("Order %d: floor %d, hall up button. \n", i, order.Floor)
		case elevio.BT_HallDown:
			fmt.Printf("Order %d: floor %d, hall down button. \n", i, order.Floor)
		case elevio.BT_Cab:
			fmt.Printf("Order %d: floor %d, cab button. \n", i, order.Floor)
		}
	}
}

// Finds lowest floor order from queue between current task (first order in queue) and current possition (lower bound)
func (q *Queue) Find_lowest_between(lower_bound int) int {
	lowest := q.Orders[0].Floor
	for _, order := range q.Orders[1:] {
		if (order.Floor < lowest && order.Floor > lower_bound) && (order.Button == elevio.BT_HallUp || order.Button == elevio.BT_Cab) {
			lowest = order.Floor
		}
	}
	return lowest
}

// Finds highest floor order from queue between current task (first order in queue) and current possition (upper bound)
func (q *Queue) Find_highest_between(upper_bound int) int {
	highest := q.Orders[0].Floor
	for _, order := range q.Orders[1:] {
		if (order.Floor > highest && order.Floor < upper_bound) && (order.Button == elevio.BT_HallDown || order.Button == elevio.BT_Cab) {
			highest = order.Floor
		}
	}
	return highest
}
