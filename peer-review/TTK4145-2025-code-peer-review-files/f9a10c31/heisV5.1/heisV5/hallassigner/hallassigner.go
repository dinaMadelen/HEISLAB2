package hallassigner

import (
	"encoding/json"
	"heisV5/synchronizer"
	"heisV5/types"
	"os/exec"
	"runtime"
	"strconv"
)

func RunHRA(cs synchronizer.SystemState, id int) types.Orders {

	stateMap := make(map[string]types.HallRequestAssignerState)
	for i, v := range cs.States {
		stateMap[strconv.Itoa(i)] = types.HallRequestAssignerState{
			Behaviour:   v.State.Activity.ToString(),
			Floor:       v.State.CurrentFloor,
			Direction:   v.State.MovingDirection.ToString(),
			CabRequests: v.CabRequests,
		}
	}
	inputToHRA := types.HallRequestAssignerDesiredInput{
		HallRequests: cs.HallRequests,
		States:       stateMap,
	}
	inputBytes, err := json.Marshal(inputToHRA)
	if err != nil {
		panic(err)
	}

	Operatingsystem := ""
	switch runtime.GOOS {
	case "linux":
		Operatingsystem = "hall_request_assigner"
	case "windows":
		Operatingsystem = "hall_request_assigner.exe"
	default:
		panic("not supported OS")
	}

	ret, err := exec.Command("distributor/executables/"+Operatingsystem, "-i", "--includeCab", string(inputBytes)).CombinedOutput()
	if err != nil {
		panic(err)
	}

	output := new(map[string]types.Orders)
	err = json.Unmarshal(ret, output)
	if err != nil {
		panic(err)
	}

	return (*output)[strconv.Itoa(id)]
}
