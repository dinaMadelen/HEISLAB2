package elevator

import (
	"Driver-go/config"
	"Driver-go/elevio"
)

type Orders [config.NumFloors][config.NumButtons]bool

func (a Orders) OrderInDirection(floor int, dir Direction) bool {
	switch dir {
	case Up:
		for f := floor + 1; f < config.NumFloors; f++ {
			for b := 0; b < config.NumButtons; b++ {
				if a[f][b] {
					return true
				}
			}
		}
		return false
	case Down:
		for f := 0; f < floor; f++ {
			for b := 0; b < config.NumButtons; b++ {
				if a[f][b] {
					return true
				}
			}
		}
		return false
	default:
		panic("Invalid direction")
	}
}

func OrderDone(floor int, dir Direction, a *Orders, orderDoneC chan<- elevio.ButtonEvent) {
	if a[floor][elevio.BT_Cab] { // Checks cab
		orderDoneC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_Cab}
		a[floor][elevio.BT_Cab] = false // Nullifies the order
	}

	button := dir.ToButtonType()
	if a[floor][button] { // Sjekker hall-knapp i retningen heisen går
		orderDoneC <- elevio.ButtonEvent{Floor: floor, Button: button}
		a[floor][button] = false // Nullifies the order
	}
}
