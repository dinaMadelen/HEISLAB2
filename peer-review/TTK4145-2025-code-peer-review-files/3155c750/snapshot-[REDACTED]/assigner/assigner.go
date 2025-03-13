package assigner

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"runtime"

)

// Struct members must be public in order to be accessible by json.Marshal/.Unmarshal
// This means they must start with a capital letter, so we need to use field renaming struct tags to make them camelCase
var elevator_id = "A"
type Elevator_data struct {//data struct that contains all the data that the assigner needs to know about the elevator 
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`    
}

type HRAInput struct {
	HallRequests [][2]bool               `json:"hallRequests"`
	States       map[string]Elevator_data `json:"states"`
}



func Assigner(localelvator Elevator_data,RemoteElevatorData Elevator_data, hallRequests [][2]bool) [][2]bool{
	var input HRAInput
	hraExecutable := ""
	switch runtime.GOOS {
	case "linux":
		hraExecutable = "hall_request_assigner"
	case "windows":
		hraExecutable = "hall_request_assigner.exe"
	default:
		panic("OS not supported")
	}
	switch elevator_id{
	case "A":
		input = HRAInput{
		HallRequests: hallRequests,
		States: map[string]Elevator_data{
			"A": localelvator,
			"B": RemoteElevatorData,
		},}
	case "B":
		input = HRAInput{
			HallRequests: hallRequests,
			States: map[string]Elevator_data{
				"A": RemoteElevatorData ,
				"B": localelvator,
			},}
	}
	jsonBytes, err := json.Marshal(input)
	if err != nil {
		fmt.Println("json.Marshal error: ", err)
		return nil
	}

	ret, err := exec.Command("assignerExecutables/" + hraExecutable, "-i", "--includeCab", string(jsonBytes)).CombinedOutput()
	if err != nil {
		fmt.Println("exec.Command error: ", err)
		fmt.Println(string(ret))
		return nil
	}

	output := make((map[string][][2]bool))
	err = json.Unmarshal(ret, &output)
	if err != nil {
		fmt.Println("json.Unmarshal error: ", err)
		return nil
	}
	
	fmt.Printf("output: \n")
	for k, v := range output {
		fmt.Printf("%6v :  %+v\n", k, v)
	}
	
	return output[elevator_id]
}
