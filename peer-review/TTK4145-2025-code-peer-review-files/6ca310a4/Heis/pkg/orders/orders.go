package orders

import (
	"Heis/pkg/elevator"
	"Heis/pkg/elevio"
	"Heis/pkg/msgTypes"
	"fmt"
	"sort"
	"reflect"
)



//temporarly, should be loaded from config
const N_floors = 4
const N_buttons = 3


// "Main" function for orders. Takes a ButtonEvent from fsm on localRequest channel when a button is pushed
// and sends an ButtonEvent on localAssignedOrder channel if this eleveator should take order
// Updates local assignedOrders from a remoteElevator sent on elevatorStateCh.
// Also checks if an order to be done by this elevator should be started or not
func OrderHandler(e *elevator.Elevator, remoteElevators *map[string]elevator.Elevator, 
	localAssignedOrder, localRequest chan elevio.ButtonEvent, elevatorStateCh chan msgTypes.ElevatorStateMsg, completedOrderCH chan elevio.ButtonEvent) {
	var activeLocalOrders [N_floors][N_buttons]bool
	for {
		select {
		case btn_input := <- localRequest:
			assignOrder(e, *remoteElevators, btn_input)

		// sent in help functions in FSM when an order is completed
		case completed_order := <- completedOrderCH:
			temp := (*e).AssignedOrders[(*e).Id]
			temp[completed_order.Floor][completed_order.Button] = elevator.Confirmed
			(*e).AssignedOrders[(*e).Id] = temp
		
		case remoteElevatorState := <-elevatorStateCh:
			if remoteElevatorState.Id != (*e).Id {
				updateFromRemoteElevator(remoteElevators, e, remoteElevatorState)
				if assignedOrdersCheck(*remoteElevators, *e){
					orderMerger(e, *remoteElevators)
				}
				fmt.Println("Local: ", (*e).AssignedOrders)
				fmt.Println("Remote: ", remoteElevatorState.Elevator.AssignedOrders)
			}
		// case for disconnection or timout for elevator to reassign orders
		// case for synchronization after restart/connection to nettwork
		}

		// Check if an unstarted assigned order should be started
		for floor := range N_floors {
			for btn := range N_buttons {
				if (*e).AssignedOrders[(*e).Id][floor][btn] != elevator.Confirmed {
					activeLocalOrders[floor][btn] = false
				}
				if assignedOrdersCheck(*remoteElevators, *e){
					if shouldStartLocalOrder(e, *remoteElevators, (*e).Id, floor, btn) && !activeLocalOrders[floor][btn] {
						localAssignedOrder <- elevio.ButtonEvent{
							Floor:  floor,
							Button: elevio.ButtonType(btn),
						}
						activeLocalOrders[floor][btn] = true
					}
				}
			}
		}
	}
}

// Updates state in assignedOrders for local elevator if it is behind remoteElevators
func orderMerger(e *elevator.Elevator, remoteElevators map[string]elevator.Elevator) {
	var currentState elevator.RequestState
	var updateState elevator.RequestState

	for id, _ := range (*e).AssignedOrders {	
		for floor := range N_floors{
			for btn := range N_buttons {
				for _, elev := range remoteElevators {
					currentState = (*e).AssignedOrders[id][floor][btn]
					updateState = elev.AssignedOrders[id][floor][btn]

					if updateState - currentState == 1 || (updateState == elevator.None && currentState == elevator.Complete) {
						temp := (*e).AssignedOrders[id]
						temp[floor][btn] = updateState
						(*e).AssignedOrders[id] = temp
					}
				}	
				confirmOrCloseOrders(e, remoteElevators, id, floor, btn)
			}
		}
	}
}

// Check if all nodes on nettwork are synced on certain order
func ordersSynced(e *elevator.Elevator, remoteElevators map[string]elevator.Elevator, id string, floor, btn int) bool {
	for _, elev := range remoteElevators {
		if (*e).AssignedOrders[id][floor][btn] != elev.AssignedOrders[id][floor][btn] {
			return false
		}
	}
	return true
}


// Increments cyclic counter if all nodes on nettwork are synced on order and state is order or completed
func confirmOrCloseOrders(e *elevator.Elevator, remoteElevators map[string]elevator.Elevator, id string, floor, btn int) {
	if ordersSynced(e, remoteElevators, id, floor, btn) {
		if  (*e).AssignedOrders[id][floor][btn] == elevator.Order {
			temp := (*e).AssignedOrders[id]
			temp[floor][btn] = elevator.Confirmed
			(*e).AssignedOrders[id] = temp
		}
		if  (*e).AssignedOrders[id][floor][btn] == elevator.Complete{
			temp := (*e).AssignedOrders[id]
			temp[floor][btn] = elevator.None
			(*e).AssignedOrders[id] = temp
		}
	}
}

// Should start order when elevators are synced and order is confirmed
func shouldStartLocalOrder(e *elevator.Elevator, remoteElevators map[string]elevator.Elevator, id string, floor, btn int) bool {
	return ordersSynced(e, remoteElevators, id, floor, btn) && (*e).AssignedOrders[id][floor][btn] == elevator.Confirmed
}

// Updates remoteElevator map and adds entries for remote Elevator in assignedOrders map for local elevator if they do not exist yet. 
func updateFromRemoteElevator(remoteElevators * map[string]elevator.Elevator, e * elevator.Elevator, remoteElevatorState msgTypes.ElevatorStateMsg) {
	remoteElevator := remoteElevatorState.Elevator
	(*remoteElevators)[remoteElevatorState.Id] = remoteElevator
	
	for id, _ := range *remoteElevators {
		_, exists := (*e).AssignedOrders[id]
		if !exists {
			(*e).AssignedOrders[id] = remoteElevator.AssignedOrders[remoteElevatorState.Id]
		}
	}
}

// Checks if all active elevators in remoteElevators have an assignedOrders map with keys for all active elevators on nettwork 
func assignedOrdersCheck(remoteElevators map[string]elevator.Elevator, elevator elevator.Elevator) bool {
	
	var localKeys []string
	var remoteKeys []string
	
	for k, _ := range elevator.AssignedOrders{
		localKeys = append(localKeys, k)
	}
	
	for _, elev := range remoteElevators {
		remoteKeys = []string{}
		for k, _ := range elev.AssignedOrders{
			remoteKeys = append(remoteKeys, k)
		}
		sort.Strings(localKeys)
		sort.Strings(remoteKeys)
		// fmt.Println("local keys", localKeys)
		// fmt.Println("External keys", remoteKeys)

		if !reflect.DeepEqual(localKeys, remoteKeys) {
			return false
		}
	}
	return true
}