package assignment

import (
	"elevatorsystem/constants"
	"elevatorsystem/single-elevator/Driver-go/elevio"
	"elevatorsystem/single-elevator/elevatorLogic"
	"encoding/json"
	"fmt"
	"os/exec"
)

type HRAElevState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type HRAInput struct {
	HallRequests [][2]bool               `json:"hallRequests"`
	States       map[string]HRAElevState `json:"states"`
}

var behaviorMap = map[int]string{
	0: "idle",
	1: "moving",
	2: "doorOpen",
}

var directionMap = map[int]string{
	1:  "up",
	-1: "down",
	0:  "stop",
}

func Assign(peerAliveList []string, elevatorDataMessageList []elevatorLogic.Elevator, newOrder elevio.ButtonEvent) map[string][][2]bool {
	// Create a map of HRAElevState for all elevators
	states := make(map[string]HRAElevState)

	// Create a map for faster lookups of peerAliveList
	peerAliveMap := make(map[string]bool)
	for _, id := range peerAliveList {
		peerAliveMap[id] = true
	}

	// Convert elevator data to HRAElevState
	for _, e := range elevatorDataMessageList {
		hraElevState := HRAElevState{
			Behavior:    behaviorMap[int(e.Behaviour)],
			Floor:       e.LastKnownFloor,
			Direction:   directionMap[int(e.Direction)],
			CabRequests: []bool{},
		}
		for floor := 0; floor < constants.NUM_FLOORS; floor++ {
			hraElevState.CabRequests = append(hraElevState.CabRequests, e.Orders[floor][elevio.BT_Cab])
		}
		if _, exists := peerAliveMap[e.ElevatorID]; exists {
			states[e.ElevatorID] = hraElevState
		}
	}

	input := HRAInput{
		HallRequests: make([][2]bool, constants.NUM_FLOORS),
		States:       states,
	}

	input.HallRequests[newOrder.Floor][newOrder.Button] = true

	// Automatic process for running the hall_request_assigner-file, but we do it manually because it is easier
	// hraExecutable := ""
	// switch runtime.GOOS {
	//     case "linux":   hraExecutable  = "hall_request_assigner"
	//     case "windows": hraExecutable  = "../assignment/hall_request_assigner.exe"
	//     default:        panic("OS not supported")
	// }

	jsonBytes, err := json.Marshal(input)
	if err != nil {
		fmt.Println("json.Marshal error: ", err)
		return make(map[string][][2]bool)
	}

	// Use this for linux. Add your path to "hall_request_assigner"
	ret, err := exec.Command("ANONYMISERT:)", "-i", string(jsonBytes)).CombinedOutput()

	// Use this for windows. Add your path to "hall_request_assigner"
	// ret, err := exec.Command("ANONYMISERT:)", "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {
		fmt.Println("exec.Command error: ", err)
		fmt.Println(string(ret))
		return make(map[string][][2]bool)
	}

	output := new(map[string][][2]bool)
	err = json.Unmarshal(ret, &output)
	if err != nil {
		fmt.Println("json.Unmarshal error: ", err)
		return make(map[string][][2]bool)
	}

	fmt.Printf("output: \n")
	for k, v := range *output {
		fmt.Printf("%6v :  %+v\n", k, v)
	}

	// Send output to distribution
	return *output
}
