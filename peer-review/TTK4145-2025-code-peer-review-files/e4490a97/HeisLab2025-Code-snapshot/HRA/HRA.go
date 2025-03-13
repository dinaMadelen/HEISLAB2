package HRA

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"runtime"
	"time"

	"github.com//HeisLab2025/nettverk"
)

//Sending elevator world-view to HRA and sending HRA-output back
func HRAMain(HRAOut chan map[string][][2]bool) {

	hraExecutable := ""
	switch runtime.GOOS {
	case "linux":
		hraExecutable = "hall_request_assigner"
	case "windows":
		hraExecutable = "hall_request_assigner.exe"
	default:
		panic("OS not supported")
	}

	for {

		time.Sleep(4000 * time.Millisecond)

		var input nettverk.HRAInput
		input.States = make(map[string]nettverk.HRAElevState)

		for key := range nettverk.InfoMap {
			elevstate := nettverk.InfoMap[key].State
			input.States[key] = elevstate
			input.HallRequests = nettverk.InfoMap[key].HallRequests
		}

		if len(nettverk.InfoMap) > 0 {
			jsonBytes, err := json.Marshal(input)
			if err != nil {
				fmt.Println("json.Marshal error: ", err)
				return
			}

			ret, err := exec.Command("./HRA/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
			if err != nil {
				fmt.Println("exec.Command error: ", err)
				fmt.Println(string(ret))
				return
			}

			output := new(map[string][][2]bool)
			err = json.Unmarshal(ret, &output)
			if err != nil {
				fmt.Println("json.Unmarshal error: ", err)
				return
			}
			HRAOut <- *output //send HRA-output 
			fmt.Printf("output: \n")
			for k, v := range *output {
				fmt.Printf("%6v :  %+v\n", k, v)
			}
		}
	}
}
