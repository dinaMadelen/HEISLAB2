package requestdistribution

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/message"
	"elevproj/config"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"runtime"
)

var _initializedMaster = false

type HRAInput struct {
	HallRequests [config.N_floors][2]bool `json:"hallRequests"`
	States       map[string]ElevatorState `json:"states"`
}

type ElevatorState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

func toElevatorState(report message.ElevatorReport) ElevatorState {
	var estate ElevatorState
	estate.Behavior = elevator.Eb_toString(report.Behaviour)
	estate.Floor = report.Floor
	estate.Direction = elevator.Ed_toString(report.Dirn)
	var cab []bool
	for i := range report.Requests {
		cab = append(cab, report.Requests[i][2])
	}
	estate.CabRequests = cab
	return estate
}

func RunHRA(fullHallRequests [config.N_floors][2]bool, elevatorStates map[string]message.ElevatorReport) map[string][config.N_floors][2]bool {
	errRet := make(map[string][config.N_floors][2]bool)
	hraExecutable := ""
	switch runtime.GOOS {
	case "linux":
		hraExecutable = "hall_request_assigner"
	case "windows":
		hraExecutable = "hall_request_assigner.exe"
	default:
		panic("OS not supported")
	}

	inputMap := make(map[string]ElevatorState)
	for i, elevator := range elevatorStates {
		inputMap[i] = toElevatorState(elevator)
	}

	input := HRAInput{HallRequests: fullHallRequests, States: inputMap}

	jsonBytes, err := json.Marshal(input)
	if err != nil {
		fmt.Println("json.Marshal error: ", err)
		return errRet
	}

	path, _ := os.Getwd()
	//fmt.Println(path)
	ret, err := exec.Command(path+"/Master/requestDistribution/cost_fns/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {
		fmt.Println("exec.Command error: ", err)
		fmt.Println(string(ret))
		return errRet
	}

	output := new(map[string][config.N_floors][2]bool)
	err = json.Unmarshal(ret, &output)
	if err != nil {
		fmt.Println("json.Unmarshal error: ", err)
		return errRet
	}

	return *output
}

func CheckFulfilledRequests(message message.ReportMessage, masterFullRequest [config.N_floors][2]bool) [config.N_floors][2]bool {
	elevatorCalls := message.Report.Requests
	for i, floor := range elevatorCalls {
		for j, btnPress := range floor {
			if j < 2 {
				if !btnPress && masterFullRequest[i][j] && message.Report.Floor == i && message.Report.Behaviour != elevator.EB_moving {
					masterFullRequest[i][j] = false
				}
			}
		}
	}
	return masterFullRequest
}
