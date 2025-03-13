package orders

import (

	"Heis/pkg/elevator"
	"Heis/pkg/elevio"

	//"fmt"
	//"strconv"
)

// func ReassignOrders(elevators []*elevator.Elevator, ch_newLoacalOrder chan elevio.ButtonEvent) {
// 	lowestID := 999

// 	for _, elev := range elevators {
// 		if elev.Behave != elevator.Unavailable {
// 			ID, _ := strconv.Atoi(elev.ID)
// 			if ID < lowestID {
// 				lowestID = ID
// 			}
// 		}
// 	}
// 	for _, elev := range elevators {
// 		if elev.Behave == elevator.Unavailable {
// 			for floor := range elev.(*e).AssignedOrders {
// 				for button := 0; button < len(elev.(*e).AssignedOrders[floor])-1; button++ {
// 					if elev.(*e).AssignedOrders[floor][button] == elevator.Order ||
// 						elev.(*e).AssignedOrders[floor][button] == elevator.Comfirmed {
// 						if elevators[elevator.LocalElevator].ID == strconv.Itoa(lowestID) {
// 							ch_newLoacalOrder <- elevio.ButtonEvent{
// 								Floor:  floor,
// 								Button: elevio.ButtonType(button),
// 							}
// 						}
// 					}
// 				}
// 			}
// 		}
// 	}
// }

// func ReassignOrders(elevators map[string]elevator.Elevator, ch_newLoacalOrder chan elevio.ButtonEvent) {
// 	lowestID := 999

// 	for _, elev := range elevators {
// 		if elev.Behaviour != elevator.Unavailable {
// 			ID, _ := strconv.Atoi(elev.Id)
// 			if ID < lowestID {
// 				lowestID = ID
// 			}
// 		}
// 	}
// 	for index, elev := range elevators {
// 		if elev.Behaviour == elevator.Unavailable {
// 			for floor := range elev.(*e).AssignedOrders[index] {
// 				for button := 0; button < len(elev.(*e).AssignedOrders[index][floor])-1; button++ {
// 					if elev.(*e).AssignedOrders[index][floor][button] == elevator.Order ||
// 						elev.(*e).AssignedOrders[index][floor][button] == elevator.Comfirmed {
// 						if elevators[elevator.LocalElevator].ID == strconv.Itoa(lowestID) {
// 							ch_newLoacalOrder <- elevio.ButtonEvent{
// 								Floor:  floor,
// 								Button: elevio.ButtonType(button),
// 							}
// 						}
// 					}
// 				}
// 			}
// 		}
// 	}
// }



func assignOrder(e * elevator.Elevator, remoteElevators map[string]elevator.Elevator, order elevio.ButtonEvent) {
	
	
	if len((*e).AssignedOrders) < 2 || order.Button == elevio.BT_Cab {
		if (*e).AssignedOrders[(*e).Id][order.Floor][order.Button] == elevator.None && ordersSynced(e, remoteElevators, (*e).Id, order.Floor, int(order.Button)){
			temp := (*e).AssignedOrders[(*e).Id]
			temp[order.Floor][order.Button] = elevator.Order
			(*e).AssignedOrders[(*e).Id] = temp
		}
		return
	}
	minCost := 99999
	elevCost := cost(*e, order)
	minCost = elevCost
	minElev := (*e).Id
	for id, elev := range remoteElevators {
		elevCost = cost(elev, order)
		if elevCost < minCost {
			minCost = elevCost
			minElev = id
		}
	}
	if (*e).AssignedOrders[minElev][order.Floor][order.Button] == elevator.None && ordersSynced(e, remoteElevators, minElev, order.Floor, int(order.Button)){
		temp := (*e).AssignedOrders[minElev]
		temp[order.Floor][order.Button] = elevator.Order
		(*e).AssignedOrders[minElev] = temp
	}
}
