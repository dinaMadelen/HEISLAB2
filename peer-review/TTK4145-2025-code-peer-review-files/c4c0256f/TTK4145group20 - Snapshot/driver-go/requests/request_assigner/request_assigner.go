/*
	This package is basically a bridge between the code and the external hall_request_assigner program,
	which acts as the brain for assigning requests in a multi-elevator system.
		- The code collects the local state and peer states.
		- It packages that into JSON.
		- It sends that to hall_request_assigner, which calculates the optimal assignments.
		- It takes the output and returns only the assignments for this elevator.
*/

package request_assigner

import (
	. "Driver-go/elevator/types"
)
type ElevState struct {
	Behaviour   string
	Floor       int
	Direction   string
	CabRequests [N_FLOORS]bool
}

type Input struct {
	HallRequests [N_FLOORS][N_HALL_BUTTONS]bool
	States       map[string]ElevState
}

/*
	- Collect information about the current state of all elevators (yours and your peers').
	- Convert the data into a format that an external executable (called hall_request_assigner) can understand.
	- Run the hall_request_assigner program, which calculates which elevator should handle which requests.
	- Take the output from hall_request_assigner and return the assigned requests for the local elevator (the one running this code).
*/

func RequestAssigner(
	hallRequests [N_FLOORS][N_HALL_BUTTONS]Request, // 2D array representing hall button requests (each floor has 2 buttons: UP and DOWN).
	allCabRequests map[string][N_FLOORS]Request, // Map of cab requests for each elevator. Each elevator has its own requests (like floor buttons inside the elevator).
	latestInfoElevators map[string]ElevatorInfo, // Latest known state (floor, behavior, direction) for each elevator.
	peerList []string, // List of known peers (other elevators in the system).
	localID string, // The ID of this elevator (the one running the code).
) [N_FLOORS][N_BUTTONS]bool {

	/*
		This part converts the hallRequests array (Request) into a simpler [N_FLOORS][N_HALL_BUTTONS]bool array:
			- It only marks true for requests that are already ASSIGNED.
			- This simplifies data for the external program.
	*/
	boolHallRequests := [N_FLOORS][N_HALL_BUTTONS]bool{}
	for floor := 0; floor < N_FLOORS; floor++ {
		for button := 0; button < N_HALL_BUTTONS; button++ {
			if hallRequests[floor][button].State == ASSIGNED {
				boolHallRequests[floor][button] = true
			}
		}
	}

	/*
		This part gathers the current state of all elevators into a format that the hall_request_assigner expects:
			- inputStates is a map, where the key is the elevator ID, and the value is an HRAElevState struct.
			- It loops over all elevators (both local and peers).
			- It skips elevators that:
				- Are not known in latestInfoElevators.
				- Are marked as unavailable.
				- Are not in the peerList (unless it's the local elevator itself).
	*/
	inputStates := map[string]ElevState{}

	for id, cabRequests := range allCabRequests {
		elevatorInfo, exists := latestInfoElevators[id]
		if !exists {
			continue
		}

		if !elevatorInfo.Available {
			continue
		}

		if !sliceContains(peerList, id) && id != localID {
			continue
		}

		boolCabRequests := [N_FLOORS]bool{}
		for floor := 0; floor < N_FLOORS; floor++ {
			if cabRequests[floor].State == ASSIGNED {
				boolCabRequests[floor] = true
			}
		}
		// This is the core of how the elevator state is packaged for the external process.
		
		inputStates[id] = ElevState{
			Behaviour:    behaviourToString(elevatorInfo.Behaviour),
			Floor:       elevatorInfo.Floor,
			Direction:   directionToString(elevatorInfo.Direction),
			CabRequests: boolCabRequests,
		}

	}

	if len(inputStates) == 0 {
		return [N_FLOORS][N_BUTTONS]bool{}
	}

	// If there are no valid elevators, it returns an empty request set.
	input := Input{ // The HRAInput struct holds the hall requests and all elevator states.
		HallRequests: boolHallRequests,
		States:       inputStates,
	}

	assignedRequest := assign_requests(input)
	sliceAssignment := assignedRequest[localID]

	var fixedAssignment [4][3]bool
	// Copy elements from sliceAssignment into fixedAssignment
	for i := 0; i < 4 && i < len(sliceAssignment); i++ {
		for j := 0; j < 3 && j < len(sliceAssignment[i]); j++ {
			fixedAssignment[i][j] = sliceAssignment[i][j]
		}
	}
	return fixedAssignment
}


func behaviourToString(b ElevBehaviour) string {
	switch b {
	case EB_Idle:
		return "idle"
	case EB_Moving:
		return "moving"
	case EB_DoorOpen:
		return "doorOpen"
	}
	return "idle"
}

func directionToString(d ElevDirection) string {
	switch d {
	case ED_Down:
		return "down"
	case ED_Up:
		return "up"
	case ED_Stop:
		return "stop"
	}
	return "stop"
}


func sliceContains(slice []string, elem string) bool {
	for _, element := range slice {
		if element == elem {
			return true
		}
	}
	return false
}
