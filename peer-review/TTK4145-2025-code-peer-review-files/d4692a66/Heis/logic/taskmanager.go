package logic

import (
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/driver/elevio"
)


func hasOrdersAbove(floor int, requests [][]int) bool {
	for f := floor + 1; f < len(requests); f++ {
		for btn := 0; btn < 3; btn++ {
			if requests[f][btn] ==2 {
				return true
			}
		}
	}
	return false
}

func hasOrdersBelow(floor int, orders [][]int) bool {
	for f := 0; f < floor; f++ {
		for btn := 0; btn < 3; btn++ {
			if orders[f][btn] == 2{
				return true
			}
		}
	}
	return false
}

func hasUnconirmedOrdersAt(floor int, orders [][]int) bool {
	for btn := 0; btn < 3; btn++ {
		if orders[floor][btn] == 1 {
			return true
		}
	}
	return false
}


func ClearRequestsAtFloor(floor int, currentDir elevio.MotorDirection, orders [][]int) {

	//for å cleare må vi ta inn pekere til globalstate og local trur eg. ordersmå være en peker og trur eg? 
	// kankje vi kan sette alle disse løkkane  inn i en ekstern som sett til tre og så ne? 
	
	orders[floor][elevio.BT_Cab]  = 0
	if currentDir == elevio.MD_Up {
		orders[floor][elevio.BT_HallUp] = 0
		if !hasOrdersAbove(floor, orders) {
			orders[floor][elevio.BT_HallDown] = 0
		}
	} else if currentDir == elevio.MD_Down {
		orders[floor][elevio.BT_HallDown] = 0
		if !hasOrdersBelow(floor, orders) {
			orders[floor][elevio.BT_HallUp] = 0
		}
	} else {
		orders[floor][elevio.BT_HallUp] = 0
		orders[floor][elevio.BT_HallDown] = 0
	}
}

func AddOrder(elevator *config.Elevator, floor int, btn elevio.ButtonType, stateTX chan *config.Elevator) {
	if btn == elevio.BT_Cab {
		elevator.Requests[floor][btn] = 2
	} else {
		elevator.Requests[floor][btn] = 1
	}
	elevio.SetButtonLamp(btn, floor, true)
	UpdateButtonLights(elevator.Requests)

	stateTX <- elevator
}

func RemoveOrder(elevator *config.Elevator, floor int, btn elevio.ButtonType, stateTX chan *config.Elevator) {
	elevator.Requests[floor][btn] = 0
	elevio.SetButtonLamp(btn, floor, false)

	stateTX <- elevator
}


