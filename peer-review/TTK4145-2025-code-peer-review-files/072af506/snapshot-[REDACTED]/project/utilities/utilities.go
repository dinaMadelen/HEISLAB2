package utilities

import (
	"main/elev_algo_go/elevator"
)

type StatusMessage struct {
	Label string
	Controller_id int
	Behaviour     string
	Floor         int
	Direction     string
	Node_orders   [elevator.N_FLOORS][elevator.N_BUTTONS]bool
}

type OrderDistributionMessage struct {
	Label string
	Orderlines [3][elevator.N_FLOORS][elevator.N_BUTTONS - 1]bool
}
