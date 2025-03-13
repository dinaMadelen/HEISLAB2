package order_handler

import (
	"fmt"
	"main/elev_algo_go/elevator"
	"main/utilities"
)

// This function builds an input for our order_assigner, so that the executable hall_request_assigner accepts it.
// The input is a slice of StatusMessages from utilities (this input is yet to be made)
func BuildAssignerInputFromStatusMessages(statuses []utilities.StatusMessage) map[string]interface{} {

	hallRequests := make([][]bool, elevator.N_FLOORS) 
	for f := 0; f < elevator.N_FLOORS; f++ {
		hallRequests[f] = make([]bool, 2) 
	} 

	states := make(map[string]interface{}) 

	// based on the status of each elevator we create a map of all hallRequests and states
	for _, s := range statuses {
		cabRequests := make([]bool, elevator.N_FLOORS)

		for f := 0; f < elevator.N_FLOORS; f++ {
			hallRequests[f][0] = hallRequests[f][0] || s.Node_orders[f][0]
			hallRequests[f][1] = hallRequests[f][1] || s.Node_orders[f][1]
			
			cabRequests[f] = s.Node_orders[f][2]
		}

		key := fmt.Sprintf("%d", s.Controller_id)
		states[key] = map[string]interface{}{
			"behaviour":   s.Behaviour,
			"floor":       s.Floor,
			"direction":   s.Direction,
			"cabRequests": cabRequests,
		}
	}

	return map[string]interface{}{
		"hallRequests": hallRequests, 
		"states":       states,      
	}
}
