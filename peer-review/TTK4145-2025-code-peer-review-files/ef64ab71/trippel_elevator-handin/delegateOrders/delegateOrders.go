package delegateOrders

import (
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
)

/*
This module should delegate all orders to an elevator.
It should consider different parameters and choose the most suitable elevator
*/


func DelegateOrders(e *elevator.Elevator) {
	for {
		//This function should only be run by the master
		if !e.IsMaster {
			time.Sleep(100 * time.Millisecond)
			continue
		}

		for floor := 0; floor < driver.N_Floors; floor++ {
			for btn := 0; btn < driver.N_Buttons+(elevator.NumberOfElevators-1); btn++ {
				//assign cab orders to the respective elevator
				if btn >= driver.N_Buttons-1 {
					elevatorIndex := btn - driver.N_Buttons + 1
					elevatorID := elevator.Elevators[elevatorIndex].ID
					if e.Orders[floor][btn].Value == 2 {
						elevator.Delegated[floor][btn] = elevatorID
						elevator.Elevators[elevatorID].MyOrders[floor][driver.BT_Cab] = 1
					}
					continue
				}
				//assign hall orders to the best elevator
				if e.Orders[floor][btn].Value == 2 { //value 2 for confirmed order
					bestElevator := findBestElevator(floor, btn, e.OnlineElevators)
					if bestElevator != -1 {
						elevator.Delegated[floor][btn] = elevator.Elevators[bestElevator].ID
						elevator.Elevators[bestElevator].MyOrders[floor][btn] = 1
					}
				}
			}
		}
		time.Sleep(100 * time.Millisecond)
		e.MyOrders = elevator.Elevators[e.ID].MyOrders
	}
}

func findBestElevator(floor int, btn int, onlineElevators [elevator.NumberOfElevators]bool) int {
	//checks that the order is not assigned
	if elevator.Delegated[floor][btn] != -1 {
		return -1
	}
	bestElevator := -1
	bestScore := 10000

	for i := 0; i < elevator.NumberOfElevators; i++ {
		//check if the respective elevator is connected
		if !onlineElevators[i] {
			continue
		}

		e := elevator.Elevators[i]

		//skip elevators that are not operational
		if e.Behaviour == elevator.EB_Stop || !e.Functional {
			continue
		}

		// Calculate score for the elevator
		score := evaluateElevator(e, floor)

		// Lower score is better
		if score < bestScore {
			bestScore = score
			bestElevator = e.ID
		}
	}

	return bestElevator
}

func evaluateElevator(e elevator.Elevator, targetFloor int) int {
	distance := e.Floor - targetFloor
	if distance < 0 { //absolute value
		distance = distance * -1
	}

	score := distance * 10

	// Prioritize elevators moving in the correct direction
	if (e.Dirn == elevator.D_Up && targetFloor > e.Floor) || (e.Dirn == elevator.D_Down && targetFloor < e.Floor) {
		score -= 5
	}

	// Idle elevators are preferred
	if e.Behaviour == elevator.EB_Idle {
		score -= 10
	}

	// Prefer elevators with fewer active orders
	orderCount := countOrders(e.MyOrders)
	score += orderCount * 3

	return score
}

func countOrders(orders [driver.N_Floors][driver.N_Buttons]int) int {
	count := 0
	for i := 0; i < driver.N_Floors; i++ {
		for j := 0; j < driver.N_Buttons; j++ {
			if orders[i][j] == 1 {
				count++
			}
		}
	}
	return count
}

// This function should reassign orders from an elevator
func UnassignOrders(failedElevatorID int) {
	for floor := 0; floor < driver.N_Floors; floor++ {
		for btn := 0; btn < driver.N_Buttons-1; btn++ {
			if elevator.Delegated[floor][btn] == failedElevatorID {
				elevator.Delegated[floor][btn] = -1
			}
		}
	}
}
