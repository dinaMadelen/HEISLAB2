package assigner

import (
	. "Project/dataenums"
	"encoding/json"
	"fmt"
	"os/exec"
)

func assignOrders(payload FromNetworkToAssigner,
	nodeID int) [NFloors][NButtons]bool {
	var orderList [NFloors][NButtons]bool

	for floor := 0; floor < NFloors; floor++ {
		orderList[floor][BCab] = payload.ElevatorList[nodeID].CabRequests[floor]
	}
	if !(payload.AliveList[nodeID]) {
		return orderList
	}

	hraInput := HRAInput{
		HallRequests: [NFloors][NButtons - 1]bool{},
		States:       make(map[string]HRAElevState),
	}
	for i, alive := range payload.AliveList {

		if alive {
			elevatorID := fmt.Sprintf("elevator_%d", i)
			hraInput.States[elevatorID] = payload.ElevatorList[i]
		}
	}
	for floor := 0; floor < NFloors; floor++ {
		for btn := BHallUp; btn <= BHallDown; btn++ {
			allAssigned := true
			for i, alive := range payload.AliveList {
				if alive && payload.HallOrderList[i][floor][btn] != OrderAssigned {
					allAssigned = false
					break
				}
			}
			hraInput.HallRequests[floor][btn] = allAssigned
		}
	}

	jsonBytes, err := json.Marshal(hraInput)
	if err != nil {
		print("Failed to marshal HRAInput: %v\n", err)
		return orderList
	}
	ret, err := exec.Command("hall_request_assigner", "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {
		print("exec.Command error: %v\nOutput: %s\n", err, string(ret))
		return orderList
	}

	output := make(map[string][][2]bool)
	if err := json.Unmarshal(ret, &output); err != nil {
		print("json.Unmarshal error: %v\n", err)
		return orderList
	}
	elevatorID := fmt.Sprintf("elevator_%d", nodeID)
	if orders, ok := output[elevatorID]; ok {
		for floor := 0; floor < NFloors && floor < len(orders); floor++ {
			for btn := BHallUp; btn < BCab; btn++ {
				orderList[floor][btn] = orders[floor][btn]
			}
			orderList[floor][BCab] = hraInput.States[elevatorID].CabRequests[floor]
		}
	}

	return orderList
}
