package requests

import (
	"heisV5/elevio"
)

// **Global RequestMatrix som oppdateres fra hallassigner**
var requestMatrix [4][3]bool
var deliveredOrderC chan elevio.ButtonEvent // **Kanal for fullførte bestillinger**

func InitRequests(completedOrders chan elevio.ButtonEvent) {
	deliveredOrderC = completedOrders
}

// **Oppdaterer hele RequestMatrix med ny data fra hallassigner**
func UpdateRequestMatrix(newMatrix [4][3]bool) {
	requestMatrix = newMatrix
}

// **Returnerer nåværende RequestMatrix**
func GetRequestMatrix() [4][3]bool {
	return requestMatrix
}

// **Legger til en ny bestilling i RequestMatrix**
func AddRequest(floor int, button elevio.ButtonType) {
	requestMatrix[floor][button] = true
	elevio.SetButtonLamp(button, floor, true)
}

// **Sjekker om heisen bør stoppe i en etasje basert på RequestMatrix**
func ShouldStopAtFloor(floor int, direction elevio.MotorDirection) bool {
	switch direction {
	case elevio.MD_Up:
		return requestMatrix[floor][elevio.BT_HallUp] || requestMatrix[floor][elevio.BT_Cab] || !requestsAbove(floor)
	case elevio.MD_Down:
		return requestMatrix[floor][elevio.BT_HallDown] || requestMatrix[floor][elevio.BT_Cab] || !requestsBelow(floor)
	case elevio.MD_Stop:
		return true
	}
	return false
}

// **Fjerner en fullført bestilling fra RequestMatrix og sender oppdatering til synchronizer via `deliveredOrderC`**
func ClearRequestsAtFloor(floor int, direction elevio.MotorDirection, id int) {
	// **Fjern cab-bestilling**
	if requestMatrix[floor][elevio.BT_Cab] {
		requestMatrix[floor][elevio.BT_Cab] = false
		elevio.SetButtonLamp(elevio.BT_Cab, floor, false)
		deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_Cab}
	}

	// **Hvis i øverste eller nederste etasje, fjern begge hallbestillinger**
	if floor == 0 || floor == 3 {
		if requestMatrix[floor][elevio.BT_HallUp] {
			requestMatrix[floor][elevio.BT_HallUp] = false
			elevio.SetButtonLamp(elevio.BT_HallUp, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallUp}
		}
		if requestMatrix[floor][elevio.BT_HallDown] {
			requestMatrix[floor][elevio.BT_HallDown] = false
			elevio.SetButtonLamp(elevio.BT_HallDown, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallDown}
		}
		return
	}

	// **Fjern hallbestillingen som samsvarer med heisens retning**
	if requestMatrix[floor][elevio.BT_HallUp] && requestMatrix[floor][elevio.BT_HallDown] {
		if direction == elevio.MD_Up {
			requestMatrix[floor][elevio.BT_HallUp] = false
			elevio.SetButtonLamp(elevio.BT_HallUp, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallUp}
		} else if direction == elevio.MD_Down {
			requestMatrix[floor][elevio.BT_HallDown] = false
			elevio.SetButtonLamp(elevio.BT_HallDown, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallDown}
		}
	} else {
		if direction == elevio.MD_Up {
			requestMatrix[floor][elevio.BT_HallUp] = false
			elevio.SetButtonLamp(elevio.BT_HallUp, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallUp}
		} else if direction == elevio.MD_Down {
			requestMatrix[floor][elevio.BT_HallDown] = false
			elevio.SetButtonLamp(elevio.BT_HallDown, floor, false)
			deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallDown}
		}
	}

	// **Fjern gjenværende bestilling hvis det ikke er flere bestillinger i den retningen**
	if direction == elevio.MD_Up && !requestsAbove(floor) {
		requestMatrix[floor][elevio.BT_HallDown] = false
		elevio.SetButtonLamp(elevio.BT_HallDown, floor, false)
		deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallDown}
	} else if direction == elevio.MD_Down && !requestsBelow(floor) {
		requestMatrix[floor][elevio.BT_HallUp] = false
		elevio.SetButtonLamp(elevio.BT_HallUp, floor, false)
		deliveredOrderC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_HallUp}
	}
}

// **Velger neste kjøreretning basert på bestillinger i RequestMatrix**
func ChooseDirection(floor int, direction elevio.MotorDirection) elevio.MotorDirection {
	switch direction {
	case elevio.MD_Up:
		if requestsAbove(floor) {
			return elevio.MD_Up
		} else if requestsBelow(floor) {
			return elevio.MD_Down
		}
	case elevio.MD_Down:
		if requestsBelow(floor) {
			return elevio.MD_Down
		} else if requestsAbove(floor) {
			return elevio.MD_Up
		}
	case elevio.MD_Stop:
		if requestsAbove(floor) {
			return elevio.MD_Up
		} else if requestsBelow(floor) {
			return elevio.MD_Down
		}
	}
	return elevio.MD_Stop
}

// **Hjelpefunksjoner for å sjekke bestillinger over/under nåværende etasje**
func requestsAbove(floor int) bool {
	for f := floor + 1; f < 4; f++ {
		for btn := 0; btn < 3; btn++ {
			if requestMatrix[f][btn] {
				return true
			}
		}
	}
	return false
}

func requestsBelow(floor int) bool {
	for f := 0; f < floor; f++ {
		for btn := 0; btn < 3; btn++ {
			if requestMatrix[f][btn] {
				return true
			}
		}
	}
	return false
}
