
package hra

import "os/exec"
import "fmt"
import "encoding/json"
import "runtime"

// Struct members must be public in order to be accessible by json.Marshal/.Unmarshal
// This means they must start with a capital letter, so we need to use field renaming struct tags to make them camelCase

type HRAElevState struct {
    Behaviour    string      `json:"behaviour"`
    Floor       int         `json:"floor"` 
    Direction   string      `json:"direction"`
    CabRequests []bool      `json:"cabRequests"`
}


type HRAInput struct {
    HallRequests    [][2]bool                   `json:"hallRequests"`
    States          map[string]HRAElevState     `json:"states"`
}


//gjør om elevstate og hallrequest til riktig format til HRA exec-fil
func SendStateToHRA(allElevStates map[int]elevator.ElevStates, globalHallRequest [][2]bool) *map[string][][2]bool {
    inputFormatHRA := make(map[string]HRAElevState)
    for id, state range allElevStates{
        inputFormatHRA[fmt.Sprintf("%d", id)] = HRAElevState{
            Behaviour: state.Behaviour,
            Floor: state.Floor,
            Direction: state.Direction,
            CabRequests: state.CabRequests,
        }
    }
    input := HRAInput {
        HallRequests: globalHallRequest, 
        States: inputFormatHRA
    }

    //lager json fil
    jsonBytes, err := json.Marshal(input)
    if err != nil {
        fmt.Println("json.Marshal error: ", err)
        return nil
    }
    
    //kjører script med json fil som input 
    ret, err := exec.Command("../hall_request_assigner/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
    if err != nil {
        fmt.Println("exec.Command error: ", err)
        fmt.Println(string(ret))
        return nil
    }
    
    //output
    output := new(map[string][][2]bool)
    err = json.Unmarshal(ret, &output)
    if err != nil {
        fmt.Println("json.Unmarshal error: ", err)
        return nil
    }
    return output
}



        