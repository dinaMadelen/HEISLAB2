package queue_assigner

import (
	"encoding/json"
	"fmt"
	"os/exec"

	commontypes "realtime_systems/common_types"
	"realtime_systems/udp_heartbeat"
)

const hraExecutable string = "hall_request_assigner"

func calculateQueue(input commontypes.HRAInput) map[string][][2]bool {

	emptyReturn := make(map[string][][2]bool)

	jsonBytes, err := json.Marshal(input)
	if err != nil {
		fmt.Println("json.Marshal error: ", err)
		return emptyReturn // TODO dummy for testing??
	}

	ret, err := exec.Command("./hall_request_assigner/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {
		fmt.Println("exec.Command error: ", err)
		fmt.Println(string(ret))
		return emptyReturn // TODO dummy for testing??
	}

	output := new(map[string][][2]bool)
	err = json.Unmarshal(ret, &output)
	if err != nil {
		fmt.Println("json.Unmarshal error: ", err)
		return emptyReturn // TODO dummy for testing??
	}

	return *output
}

func StateToQueue(elevatorStates commontypes.HRAInput, aliveNodes *udp_heartbeat.NodeStatus) map[string][][2]bool {
	// Intentionally not using a reference to avoid the need of a mutex
	// This is safe because the input is not modified

	input := commontypes.HRAInput{
		HallRequests: elevatorStates.HallRequests,
		States:       make(map[string]commontypes.ElevState),
	}

	aliveNodes.Mu_an.Lock()
	for id, active := range aliveNodes.Alive_nodes {
		if active {
			input.States[id] = elevatorStates.States[id]
		}
	}
	aliveNodes.Mu_an.Unlock()

	output := calculateQueue(input)

	return output
}
