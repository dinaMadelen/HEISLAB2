package assignment

import (
	"elevator/distribution"
	"elevator/elevator_interface"
	"elevator/elevio"
	"encoding/json"
	"fmt"
	"os/exec"
	"runtime"
)

type HRAElevState struct {
	Behaviour   string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type HRAInput struct {
	HallRequests [][2]bool               `json:"hallRequests"`
	States       map[string]HRAElevState `json:"states"`
}

func orderStatusToBool(orderStatus elevator_interface.OrderStatus) bool {
	return orderStatus != elevator_interface.NoOrder
}

func extractHallOrdersFromOrders(orders [elevio.N_FLOORS][elevio.N_BUTTONS]elevator_interface.OrderStatus) [][2]bool {
	hallOrders := make([][2]bool, len(orders))
	i := 0
	for _, floor := range orders {
		hallOrders[i] = [2]bool{orderStatusToBool(floor[0]), orderStatusToBool(floor[1])} // floor[0]: UP, floor[1]: DOWN
		i++
	}
	return hallOrders
}

func extractCabOrdersFromOrders(orders [elevio.N_FLOORS][elevio.N_BUTTONS]elevator_interface.OrderStatus) []bool {
	cabOrders := make([]bool, len(orders))
	i := 0
	for _, floor := range orders {
		cabOrders[i] = orderStatusToBool(floor[2]) // floor[2]: CAB
		i++
	}
	return cabOrders
}

func formatForHallRequestAssigner(newWorldview elevator_interface.Worldview) HRAInput {
	inputToAssigner := HRAInput{}

	processedHallRequest := false
	for id, elevator := range newWorldview.Elevators {
		if !processedHallRequest {
			inputToAssigner.HallRequests = extractHallOrdersFromOrders(newWorldview.Orders[distribution.GetPersonalID()])
			processedHallRequest = true
		}
		CabRequest := extractCabOrdersFromOrders(newWorldview.Orders[id])

		formattedState := HRAElevState{
			Behaviour:   fmt.Sprintf("%v", elevator.Behaviour),
			Floor:       elevator.Floor,
			Direction:   fmt.Sprintf("%v", elevator.Dirn),
			CabRequests: CabRequest,
		}
		inputToAssigner.States[id] = formattedState
	}

	return inputToAssigner
}

func AssignOrders(newState chan elevator_interface.Worldview, updateRequests chan map[string][elevio.N_FLOORS][2]bool) {

	hraExecutable := ""
	switch runtime.GOOS {
	case "linux":
		hraExecutable = "hall_request_assigner"
	case "windows":
		hraExecutable = "hall_request_assigner.exe"
	default:
		panic("OS not supported")
	}

	var currentWorldview elevator_interface.Worldview
	for {
		currentWorldview = <-newState
		assignerInput := formatForHallRequestAssigner(currentWorldview)
		jsonBytes, err := json.Marshal(assignerInput)
		if err != nil {
			fmt.Println("json.Marshal error: ", err)
			return
		}

		ret, err := exec.Command("./hall_request_assigner/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
		if err != nil {
			fmt.Println("exec.Command error: ", err)
			fmt.Println(string(ret))
			return
		}

		newRequests := new(map[string][elevio.N_FLOORS][2]bool)
		err = json.Unmarshal(ret, &newRequests)
		if err != nil {
			fmt.Println("json.Unmarshal error: ", err)
			return
		}
		fmt.Printf("output: \n")
		for k, v := range *newRequests {
			fmt.Printf("%6v :  %+v\n", k, v)
		}

		updateRequests <- *newRequests

	}

}
