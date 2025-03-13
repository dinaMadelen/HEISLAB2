package handleOrders

import (
	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
)

/*
This module has two tasks: finding a target floor for an elevator and mark an order as completed
*/

func FindTargetFloor(e elevator.Elevator) int {
	elevator.Mutex.Lock()
	defer elevator.Mutex.Unlock()

	nextDir := chooseDirection(e)

	currentFloor := e.Floor

	//switch direction if at top or bottom floor
	if e.Floor == driver.N_Floors-1 && nextDir == elevator.D_Up {
		nextDir = elevator.D_Down
	} else if e.Floor == 0 && nextDir == elevator.D_Down {
		nextDir = elevator.D_Up
	}

	switch nextDir {
	case elevator.D_Up:
		for i := currentFloor; i < driver.N_Floors; i++ { //prioritize orders in same direction or cab
			if e.MyOrders[i][driver.BT_HallUp] == 1 || e.MyOrders[i][driver.BT_Cab] == 1 {
				return i
			}
		}
		for i := currentFloor; i < driver.N_Floors; i++ {
			if e.MyOrders[i][driver.BT_HallDown] == 1 {
				return i
			}

		}
	case elevator.D_Down:
		for i := currentFloor; i > -1; i-- { //prioritize orders in same direction or cab
			if e.MyOrders[i][driver.BT_HallDown] == 1 || e.MyOrders[i][driver.BT_Cab] == 1 {
				return i
			}
		}
		for i := currentFloor; i > -1; i-- {
			if e.MyOrders[i][driver.BT_HallUp] == 1 {
				return i
			}
		}
	case elevator.D_Stop:
		for i := 0; i < driver.N_Buttons; i++ {
			if e.MyOrders[e.Floor][i] == 1 {
				return i
			}

		}
	}
	return -1
}

func orders_above(e elevator.Elevator) bool {
	for i := e.Floor + 1; i < driver.N_Floors; i++ {
		for j := 0; j < driver.N_Buttons; j++ {
			if e.MyOrders[i][j] == 1 {
				return true
			}
		}
	}
	return false
}

func orders_below(e elevator.Elevator) bool {
	for i := 0; i < e.Floor; i++ {
		for j := 0; j < driver.N_Buttons; j++ {
			if e.MyOrders[i][j] == 1 {
				return true
			}
		}
	}
	return false
}

func orders_here(e elevator.Elevator) bool {
	for i := 0; i < driver.N_Buttons; i++ {
		if e.MyOrders[e.Floor][i] == 1 {
			return true
		}
	}
	return false
}

func chooseDirection(e elevator.Elevator) elevator.Dirn {
	switch e.Dirn {
	case elevator.D_Up:
		if orders_above(e) {
			return elevator.D_Up
		} else if orders_here(e) {
			return elevator.D_Up
		} else if orders_below(e) {
			return elevator.D_Down
		} else {
			return elevator.D_Stop
		}

	case elevator.D_Down:
		if orders_below(e) {
			return elevator.D_Down
		} else if orders_here(e) {
			return elevator.D_Down
		} else if orders_above(e) {
			return elevator.D_Up
		} else {
			return elevator.D_Stop
		}

	case elevator.D_Stop:
		if orders_here(e) {
			return elevator.D_Stop
		} else if orders_above(e) {
			return elevator.D_Up
		} else if orders_below(e) {
			return elevator.D_Down
		} else {
			return elevator.D_Stop
		}

	default:
		return elevator.D_Stop
	}
}

// mark an order as completed in both myorders, orders and delegated
func ClearOrders(e *elevator.Elevator, floor int) {
	elevator.Mutex.Lock()
	defer elevator.Mutex.Unlock()

	switch e.Dirn {
	case elevator.D_Stop:

		for i := 0; i < driver.N_Buttons-1; i++ {
			if e.MyOrders[floor][i] == 1 {
				setOrderComplete(e, floor, i)
			}
		}

	case elevator.D_Up:

		if e.MyOrders[floor][driver.BT_HallUp] == 1 { //prioritize delete orders in same direction
			setOrderComplete(e, floor, int(driver.BT_HallUp))

		} else if e.MyOrders[floor][driver.BT_HallDown] == 1 && !orders_above(*e) {
			setOrderComplete(e, floor, int(driver.BT_HallDown))
		}

		if floor == driver.N_Floors-1 { //edge case, turn direction at top floor
			if e.MyOrders[floor][driver.BT_HallDown] == 1 {
				setOrderComplete(e, floor, int(driver.BT_HallDown))
			}
		}

	case elevator.D_Down:

		if e.MyOrders[floor][driver.BT_HallDown] == 1 { //prioritize delete orders in same direction
			setOrderComplete(e, floor, int(driver.BT_HallDown))

		} else if e.MyOrders[floor][driver.BT_HallUp] == 1 && !orders_below(*e) {
			setOrderComplete(e, floor, int(driver.BT_HallUp))
		}

		if floor == 0 {
			if e.MyOrders[floor][driver.BT_HallUp] == 1 { //edge case, turn direction at bottom floor
				setOrderComplete(e, floor, int(driver.BT_HallUp))
			}
		}
	}

	// Clear cab orders for the given floor
	if e.MyOrders[floor][driver.BT_Cab] == 1 {
		setOrderComplete(e, floor, int(driver.BT_Cab)+e.ID, true)
	}

	elevator.Elevators[e.ID] = *e
}

// updates the values for orders, myorders and delegated
func setOrderComplete(e *elevator.Elevator, floor int, button int, isCab ...bool) {

	e.Orders[floor][button].Value = 3
	e.Orders[floor][button].List = append(e.Orders[floor][button].List, e.ID)

	elevator.Delegated[floor][button] = -2
	if len(isCab) > 0 {
		button = button - e.ID
	}
	e.MyOrders[floor][button] = 0
}
