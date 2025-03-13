package mergeOrders

/*
This module is responsible for making sure all the elevators has the same worldview

An order has four different values.
	0 - No order
	1 - Unconfirmed order
	2 - Confirmed order
	3 - Something to delete

	the local elevator vil compare its own orders with the orders from one of the other elevators. And updates its own worldview
	Before an order is confirmed/deleted all elevators should acknowledge that the order is recived. All conntected elevators id's should therefore be placed
	in a list to make sure of this
*/

import (
	"math/rand"
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
)

func MergeOrders(e *elevator.Elevator) {
	for {
		time.Sleep(100 * time.Millisecond)

		selectedElevator, found := chooseElevator(*e) //random elevator that is connected, used for comparison of worldview

		switch {
		//Elevator alone in network
		case !found:
			for floor := 0; floor < driver.N_Floors; floor++ {
				for button := 0; button < driver.N_Buttons+(elevator.NumberOfElevators-1); button++ {
					if e.Orders[floor][button].Value == 3 {

						e.Orders[floor][button].Value = 0
						e.Orders[floor][button].List = []int{}

						turnOffLamp(floor, button)

					} else if e.Orders[floor][button].Value == 1 {

						if selectedElevator.Orders[floor][button].Value == 3 && !ID_inList(e.ID, selectedElevator.Orders[floor][button].List) {
							deleteOrder(e, floor, button)
						}

						turnOnLamp(floor, button)
					}
				}
			}
		case found:
			for floor := 0; floor < driver.N_Floors; floor++ {
				for button := 0; button < driver.N_Buttons+(elevator.NumberOfElevators-1); button++ {

					value := e.Orders[floor][button].Value
					switch {
					case value == 3:

						if selectedElevator.Orders[floor][button].Value == 0 {
							deleteOrder(e, floor, button)
						}

						if selectedElevator.Orders[floor][button].Value == 3 && len(selectedElevator.Orders[floor][button].List) == numberOfOnlineElevators(*e) {
							if selectedElevator.Orders[floor][button].Value == 3 && !ID_inList(e.ID, selectedElevator.Orders[floor][button].List) {
								deleteOrder(e, floor, button)
							}
						}

						//TODO: er dette riktig?
						if selectedElevator.Orders[floor][button].Value == 3 && !ID_inList(e.ID, selectedElevator.Orders[floor][button].List) {
							deleteOrder(e, floor, button) //found order to be deleted
						}

					case value == 2:
						if selectedElevator.Orders[floor][button].Value == 3 {
							foundOrderToBeDeleted(e, floor, button, selectedElevator)
						}

					case value == 1:
						if selectedElevator.Orders[floor][button].Value == 2 {
							foundConfirmedOrder(e, floor, button)
						}

						if selectedElevator.Orders[floor][button].Value == 1 && len(selectedElevator.Orders[floor][button].List) == numberOfOnlineElevators(*e) {
							foundConfirmedOrder(e, floor, button)
						}

						if selectedElevator.Orders[floor][button].Value == 1 {

							newList := union(e.Orders[floor][button].List, selectedElevator.Orders[floor][button].List)

							e.Orders[floor][button].List = newList

							if len(e.Orders[floor][button].List) == numberOfOnlineElevators(*e) {
								foundConfirmedOrder(e, floor, button)
							}
						}

					case value == 0:
						if selectedElevator.Orders[floor][button].Value == 1 {
							foundNewOrder(e, floor, button, selectedElevator)
						}
					}
				}
			}
		}
	}
}

func deleteOrder(e *elevator.Elevator, orderFloor int, orderButton int) {
	e.Orders[orderFloor][orderButton].Value = 0
	elevator.Delegated[orderFloor][orderButton] = -1

	emptyOrdersList(orderFloor, orderButton, e)
	turnOffLamp(orderFloor, orderButton)
}

func foundOrderToBeDeleted(e *elevator.Elevator, orderFloor int, orderButton int, selectedElevator elevator.Elevator) {
	e.Orders[orderFloor][orderButton].Value = 3
	elevator.Delegated[orderFloor][orderButton] = -1

	if len(selectedElevator.Orders[orderFloor][orderButton].List) > len(e.Orders[orderFloor][orderButton].List) {
		e.Orders[orderFloor][orderButton].List = append(selectedElevator.Orders[orderFloor][orderButton].List, e.ID)
	}

	if len(e.Orders[orderFloor][orderButton].List) == numberOfOnlineElevators(*e) {
		deleteOrder(e, orderFloor, orderButton)
	}
}
func foundConfirmedOrder(e *elevator.Elevator, orderFloor int, orderButton int) {
	elevator.Mutex.Lock()
	e.Orders[orderFloor][orderButton].Value = 2
	elevator.Mutex.Unlock()

	turnOnLamp(orderFloor, orderButton)
	emptyOrdersList(orderFloor, orderButton, e)
}

func foundNewOrder(e *elevator.Elevator, orderFloor int, orderButton int, selectedElevator elevator.Elevator) {

	e.Orders[orderFloor][orderButton].Value = 1

	if len(selectedElevator.Orders[orderFloor][orderButton].List) > len(e.Orders[orderFloor][orderButton].List) && !ID_inList(e.ID, selectedElevator.Orders[orderFloor][orderButton].List) {
		e.Orders[orderFloor][orderButton].List = append(selectedElevator.Orders[orderFloor][orderButton].List, e.ID)
	}

	if len(e.Orders[orderFloor][orderButton].List) == numberOfOnlineElevators(*e) {
		foundConfirmedOrder(e, orderFloor, orderButton)
	}
}

func ID_inList(ID int, list []int) bool {
	for i := 0; i < len(list); i++ {
		if list[i] == ID {
			return true
		}
	}
	return false
}

func union(slice1 []int, slice2 []int) []int {
	resultMap := make(map[int]struct{})

	for _, v := range slice1 {
		resultMap[v] = struct{}{}
	}

	for _, v := range slice2 {
		resultMap[v] = struct{}{}
	}

	var result []int
	for key := range resultMap {
		result = append(result, key)
	}

	return result
}

func emptyOrdersList(floor int, button int, e *elevator.Elevator) {
	e.Orders[floor][button].List = []int{}
}

func chooseElevator(e elevator.Elevator) (elevator.Elevator, bool) {
	var availableElevators []elevator.Elevator

	for i, isOnline := range e.OnlineElevators {
		if isOnline && i != e.ID {
			availableElevators = append(availableElevators, elevator.Elevators[i])
		}
	}

	// Return a random elevator
	if len(availableElevators) > 0 {
		rand.Seed(time.Now().UnixNano())
		randomIndex := rand.Intn(len(availableElevators))
		return availableElevators[randomIndex], true
	}
	return elevator.Elevator{}, false
}

func turnOffLamp(orderFloor int, orderButton int) {
	if driver.ButtonType(orderButton) == driver.BT_HallUp || driver.ButtonType(orderButton) == driver.BT_HallDown {
		driver.SetButtonLamp(driver.ButtonType(orderButton), orderFloor, false)
	} else if orderButton == int(driver.BT_Cab)+elevator.LocalElevator.ID {
		driver.SetButtonLamp(driver.BT_Cab, orderFloor, false)
	}
}

func turnOnLamp(orderFloor int, orderButton int) {
	if driver.ButtonType(orderButton) == driver.BT_HallUp || driver.ButtonType(orderButton) == driver.BT_HallDown {
		driver.SetButtonLamp(driver.ButtonType(orderButton), orderFloor, true)
	} else if orderButton == int(driver.BT_Cab)+elevator.LocalElevator.ID {
		driver.SetButtonLamp(driver.BT_Cab, orderFloor, true)
	}
}

func numberOfOnlineElevators(e elevator.Elevator) int {
	counter := 0
	for i := 0; i < len(e.OnlineElevators); i++ {
		if e.OnlineElevators[i] {
			counter += 1
		}
	}
	return counter
}
