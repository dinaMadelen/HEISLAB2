package distribution

import (
	"elevatorsystem/constants"
	"fmt"
)

// Not done yet. Need to implement cyclic counter logic.
func UpdateOrderStatesAllElevators(CurrentOrderListAllElevators *map[string][constants.NUM_FLOORS][constants.NUM_BUTTONS]int, ID_OfSender string, ReceivedOrderListAllElevators map[string][constants.NUM_FLOORS][constants.NUM_BUTTONS]int) {
	fmt.Println("UpdateOrderListAllElevators called")
	fmt.Println("ID_OfSender:", ID_OfSender)

	fmt.Println("CurrentOrderListAllElevators:")
	for id, orders := range *CurrentOrderListAllElevators {
		fmt.Printf("ID: %s\n", id)
		for floor := 0; floor < constants.NUM_FLOORS; floor++ {
			for button := 0; button < constants.NUM_BUTTONS; button++ {
				fmt.Printf("Floor %d, Button %d: %d\n", floor, button, orders[floor][button])
			}
		}
	}

	fmt.Println("ReceivedOrderListAllElevators:")
	for id, orders := range ReceivedOrderListAllElevators {
		fmt.Printf("ID: %s\n", id)
		for floor := 0; floor < constants.NUM_FLOORS; floor++ {
			for button := 0; button < constants.NUM_BUTTONS; button++ {
				fmt.Printf("Floor %d, Button %d: %d\n", floor, button, orders[floor][button])
			}
		}
	}
}
