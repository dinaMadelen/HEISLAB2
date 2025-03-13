package order_handler

import (
	"fmt"
	"strconv"

	"main/elev_algo_go/elevator"
	"main/utilities"
)


func OrderDistributionMessage(rawOutput map[string]interface{}) (utilities.OrderDistributionMessage, error) {
	var ODM utilities.OrderDistributionMessage 
	ODM.Label = "D" 

	
	for i := 0; i < len(ODM.Orderlines); i++ {
		key := strconv.Itoa(i)
		orders, ok := rawOutput[key].([]interface{})
		if !ok {
			// key missing(i.e an ): we let the base value (false) remain.
			continue
		}

		for j := 0; j < elevator.N_FLOORS && j < len(orders); j++ {
			floorOrders, ok := orders[j].([]interface{})
			if !ok {
				return ODM, fmt.Errorf("uventet type for nøkkel %s, etasje %d", key, j)
			}

			for k := 0; k < elevator.N_BUTTONS-1 && k < len(floorOrders); k++ {
				state, ok := floorOrders[k].(bool)
				if !ok {
					return ODM, fmt.Errorf("uventet type for nøkkel %s, etasje %d, knapp %d", key, j, k)
				}
				ODM.Orderlines[i][j][k] = state
			}
		}
	}

	fmt.Println(ODM)
	return ODM, nil
}
