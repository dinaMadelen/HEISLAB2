
package runHRA

import "os/exec"
import "fmt"
import "encoding/json"
import "runtime"
import "sanntids/cmd/localElevator/elevator"
import "sanntids/cmd/localElevator/config"

// Struct members must be public in order to be accessible by json.Marshal/.Unmarshal
// This means they must start with a capital letter, so we need to use field renaming struct tags to make them camelCase

type HRAElevState struct {
    Behavior    string      `json:"behaviour"`
    Floor       int         `json:"floor"` 
    Direction   string      `json:"direction"`
    CabRequests []bool      `json:"cabRequests"`
}

type HRAInput struct {
    HallRequests    [config.N_FLOORS][2]bool                   `json:"hallRequests"`
    States          map[string]HRAElevState                      `json:"states"`
}

func transformToElevatorState(e elevator.Elevator) HRAElevState{
	var elevstate HRAElevState
	elevstate.Behavior = elevator.Eb_toString(e.Behaviour)
	elevstate.Floor = e.Floor
	elevstate.Direction = elevator.Md_toString(e.MotorDirection)
	var cab []bool
	for i:= range e.Requests{
		cab = append(cab, e.Requests[i][2])
	}
	elevstate.CabRequests = cab
	return elevstate
}

//hallrequests: [N_floors][2]bool (opp ned)


func runHRA(hallRequests [config.N_FLOORS][2]bool, states map[string]HRAElevState) (map[string][config.N_FLOORS][2]bool){

    hraExecutable := ""
    switch runtime.GOOS {
        case "linux":   hraExecutable  = "hall_request_assigner"
        case "windows": hraExecutable  = "hall_request_assigner.exe"
        default:        panic("OS not supported")
    }
    fmt.Println(hraExecutable)
    inputMap := make (map[string]HRAElevState)
    for id, state := range states {
        inputMap[id] = state
    }

    input := HRAInput{HallRequests: hallRequests, States: inputMap}

    jsonBytes, err := json.Marshal(input)
    if err != nil {
        fmt.Println("json.Marshal error: ", err)
        return nil
    }
    //path,_:=os.Getwd();
    ret, err := exec.Command("build/" + hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
    if err != nil {
        fmt.Println("exec.Command error: ", err)
        fmt.Println(string(ret))
        return nil
    }
    
    output := make(map[string][config.N_FLOORS][2]bool)
    err = json.Unmarshal(ret, &output)
    if err != nil {
        fmt.Println("json.Unmarshal error: ", err)
        return nil
    }
        
    fmt.Printf("output: \n")
    for k, v := range output {
        fmt.Printf("%6v :  %+v\n", k, v)
    }

    return output
}

func Test()map[string][config.N_FLOORS][2]bool{

    hallRequests := [config.N_FLOORS][2]bool{{false, false}, {true, false}, {false, false}, {false, true}}

    states := map[string]HRAElevState{
        "one": HRAElevState{
            Behavior:       "moving",
            Floor:          2,
            Direction:      "up",
            CabRequests:    []bool{false, false, false, true},
        },
        "two": HRAElevState{
            Behavior:       "idle",
            Floor:          0,
            Direction:      "stop",
            CabRequests:    []bool{false, false, false, false},
        },
    }
    answer := runHRA(hallRequests, states)
    return answer
}