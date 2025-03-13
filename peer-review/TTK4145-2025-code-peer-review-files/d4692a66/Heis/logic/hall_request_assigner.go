package logic

import (
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/driver/elevio"

	"encoding/json"
	"fmt"
	"os/exec"
	"runtime"
	"time"
)

func HallRequestAssigner(hallRequests [][2]bool, states map[string]config.HRAElevState) (map[string][][2]bool, error) {

	hraExecutable := ""
	switch runtime.GOOS {
	case "linux":
		hraExecutable = "hall_request_assigner"
	case "windows":
		hraExecutable = "hall_request_assigner.exe"
	default:
		panic("OS not supported")
	}

	input := config.HRAInput{
		HallRequests: hallRequests,
		States:       states,
	}

	jsonBytes, err := json.Marshal(input)
	if err != nil {

		return nil, fmt.Errorf("json.Marshal error: ", err)
	}

	ret, err := exec.Command("Heis/Project-resources-master/cost_fns/hall_request_assigner/"+hraExecutable, "-i", string(jsonBytes)).CombinedOutput()
	if err != nil {

		return nil, fmt.Errorf("exec.Command error: %v, output: %s", err, string(ret))
	}

	var output map[string][][2]bool
	if err = json.Unmarshal(ret, &output); err != nil {
		return nil, fmt.Errorf("json.Unmarshal error: ", err)
	}

	return output, nil

}

func ElevatortoHRAELEV(elevatormap *map[string]config.Elevator) map[string]config.HRAElevState {
	hraElevMap := make(map[string]config.HRAElevState)

	// Kopierer dataen i en lokal variabel
	config.StateMutex.RLock()
	localCopy := make(map[string]config.Elevator)
	for key, value := range *elevatormap {
		localCopy[key] = value
	}
	config.StateMutex.RUnlock() // Låser opp etter kopiering

	// Leser fra lokal kopi uten låsing
	for key, value := range localCopy {
		cabRequests := make([]bool, config.NumFloors)
		for floor := 0; floor < config.NumFloors; floor++ {
			if value.Requests[floor][elevio.BT_Cab] == 2 {
				cabRequests[floor] = true
			} else {
				cabRequests[floor] = false
			}
		}

		behavior := ""
		switch value.State {
		case config.IDLE:
			behavior = "idle"
		case config.MOVING:
			behavior = "moving"
		case config.DOOR_OPEN:
			behavior = "door_open"
		case config.STOPPED:
			behavior = "stopped"
		default:
			behavior = "unknown"
		}

		direction := ""
		switch value.CurrDirn {
		case elevio.MD_Up:
			direction = "up"
		case elevio.MD_Down:
			direction = "down"
		case elevio.MD_Stop:
			direction = "stop"
		default:
			direction = "unknown"
		}

		hraElevMap[key] = config.HRAElevState{
			Behavior:    behavior,
			Floor:       value.Floor,
			Direction:   direction,
			CabRequests: cabRequests,
		}
	}

	return hraElevMap
}

func CreateHallRequests(elevators *map[string]config.Elevator) [][2]bool {
	hallRequests := make([][2]bool, config.NumFloors)
	for _, elevator := range *elevators {
		for floor := 0; floor < config.NumFloors; floor++ {
			if elevator.Requests[floor][elevio.BT_HallUp] == 1 {
				hallRequests[floor][0] = true
			}
			if elevator.Requests[floor][elevio.BT_HallDown] == 1 {
				hallRequests[floor][1] = true
			}
		}
	}
	return hallRequests
}

func SetToUnconfirmed(elevators *map[string]config.Elevator, localelevator *config.Elevator) { // om vi har en 1, settes alle til 1
	for floor := 0; floor < config.NumFloors; floor++ {
		for button := 0; button < config.NumButtons; button++ {

			for _, elevator := range *elevators {
				if elevator.Requests[floor][button] == 2 {
					break
				}
				if elevator.Requests[floor][button] == 1 {
					if localelevator.Requests[floor][button] == 0 {
						localelevator.Requests[floor][button] = 1
					}
				}

			}
		}
	}
}

func UpdateElevatorHallRequests(elevator *config.Elevator, elevators *map[string]config.Elevator) {

	HRAElevMap := ElevatortoHRAELEV(&config.GlobalState)
	HallReqs := CreateHallRequests(&config.GlobalState)
	SetToUnconfirmed(elevators, elevator)

	updatedRequests, err := HallRequestAssigner(HallReqs, HRAElevMap)
	if err != nil {
		//fmt.Printf("Feil ved kall til HallRequestAssigner: %v\n", err)
		return
	}

	for floor, requests := range updatedRequests[elevator.ID] {
		if requests[0] {
			elevator.Requests[floor][0] = 2
			//fmt.Printf("Oppdatert opp-request i etasje %d for heis %s\n", floor, elevator.ID)
			elevator.CurrDirn = ChooseDirection(elevator, elevator.CurrDirn)
			elevio.SetMotorDirection(elevator.CurrDirn)

		}
		if requests[1] {
			elevator.Requests[floor][1] = 2
			//fmt.Printf("Oppdatert ned-request i etasje %d for heis %s\n", floor, elevator.ID)
			//fmt.Printf("Elevator requests: %v\n", elevator.Requests)
			elevator.CurrDirn = ChooseDirection(elevator, elevator.CurrDirn)
			elevio.SetMotorDirection(elevator.CurrDirn)
		}
	}

	config.StateMutex.Lock()
	config.GlobalState[elevator.ID] = *elevator
	config.StateMutex.Unlock()

}

func RunHRA(elevator *config.Elevator, elevators *map[string]config.Elevator) {

	hallReqTicker := time.NewTicker(500 * time.Millisecond)
	defer hallReqTicker.Stop()

	for range hallReqTicker.C {
		UpdateElevatorHallRequests(elevator, elevators)
	}
}
func MarkRequestCompleted(elevator *config.Elevator) {
	floor := elevator.Floor

	// ✅ Ensure `BT_Cab` is marked correctly before clearing
	if elevator.Requests[floor][elevio.BT_Cab] == 2 {
		elevator.Requests[floor][elevio.BT_Cab] = 3
		//fmt.Printf("Elevator %s: Cab request at floor %d marked as completed (2 -> 3).\n", elevator.ID, floor)
	}

	// ✅ Loop through BT_HallUp and BT_HallDown only
	for _, button := range []elevio.ButtonType{elevio.BT_HallUp, elevio.BT_HallDown} {
		if elevator.Requests[floor][button] == 2 {

			// ✅ Prevent modifying invalid button positions
			if (button == elevio.BT_HallUp && floor == config.NumFloors-1) || // Top floor can't have HallUp
				(button == elevio.BT_HallDown && floor == 0) { // Bottom floor can't have HallDown
				continue // Skip this iteration
			}

			// ✅ Correctly mark hall requests based on direction
			if (button == elevio.BT_HallUp && elevator.CurrDirn == elevio.MD_Up) ||
				(button == elevio.BT_HallDown && elevator.CurrDirn == elevio.MD_Down) {
				elevator.Requests[floor][button] = 3
				//fmt.Printf("Elevator %s: Hall request (%d) at floor %d marked as completed (2 -> 3).\n",
				//	elevator.ID, button, floor)
			}
		}
	}

	// ✅ Only mark opposite direction request if it was `2` before
	if elevator.CurrDirn == elevio.MD_Up && !hasOrdersAbove(floor, elevator.Requests) {
		if floor != 0 && elevator.Requests[floor][elevio.BT_HallDown] == 2 { // ❌ Avoid setting at floor 0
			elevator.Requests[floor][elevio.BT_HallDown] = 3
			//fmt.Printf("Elevator %s: No more orders above, marking HallDown at floor %d as completed (2 -> 3).\n",
			//	elevator.ID, floor)
		}
	} else if elevator.CurrDirn == elevio.MD_Down && !hasOrdersBelow(floor, elevator.Requests) {
		if floor != config.NumFloors-1 && elevator.Requests[floor][elevio.BT_HallUp] == 2 { // ❌ Avoid setting at top floor
			elevator.Requests[floor][elevio.BT_HallUp] = 3
			//fmt.Printf("Elevator %s: No more orders below, marking HallUp at floor %d as completed (2 -> 3).\n",
			//	elevator.ID, floor)
		}
	}

	// ✅ Print updated request list
	//fmt.Printf("Elevator %s: Requests at floor %d after update: %v\n", elevator.ID, floor, elevator.Requests[floor])
}
