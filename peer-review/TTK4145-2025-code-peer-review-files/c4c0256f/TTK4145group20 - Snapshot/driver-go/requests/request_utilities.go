package request_control

import (
	. "Driver-go/elevator/types"
)

/*
This function determines whether the local elevator should accept a
request update coming from another elevator in the network.
*/

func shouldAcceptRequest(localRequest Request, messageRequest Request) bool {
	// If the remote request's count (how many times it's been updated) is lower, discard it.
	// The local request is more recent.
	if messageRequest.Count < localRequest.Count {
		return false
	}

	// If the remote request's count is higher, accept it.
	// The remote request is more up-to-date.
	if messageRequest.Count > localRequest.Count {
		return true
	}
	/*
		- Both requests are in the same state.
		- The remote request's AwareList (the list of elevators aware of the request)
		  is already fully contained in the local request's list.
		- This means the local elevator already knows everything the remote request knows.
		- In this case, there's no new information, so do NOT accept the remote request.
	*/
	if messageRequest.State == localRequest.State && IsSubset(messageRequest.AwareList, localRequest.AwareList) {
		// no new info
		return false
	}
	/*
		If the states differ (local vs. remote), it uses a priority system to decide which state "wins".
		- The priority is NEW > COMPLETED > ASSIGNED.
		Can be modified to better suit the system's needs.
	*/
	switch localRequest.State {
	case COMPLETED:
		switch messageRequest.State {
		case COMPLETED:
			return true
		case NEW:
			return true
		case ASSIGNED:
			return true
		}
	case NEW:
		switch messageRequest.State {
		case COMPLETED:
			return false
		case NEW:
			return true
		case ASSIGNED:
			return true
		}
	case ASSIGNED:
		switch messageRequest.State {
		case COMPLETED:
			return false
		case NEW:
			return false
		case ASSIGNED:
			return true
		}
	}
	print("shouldAcceptRequest() did not return")
	return false
}

// This function checks if one list of strings (a "subset") is entirely contained within another list of strings (a "superset").
func IsSubset(subset []string, superset []string) bool {
	checkset := make(map[string]bool)
	for _, element := range subset {
		checkset[element] = true
	}
	for _, value := range superset {
		if checkset[value] {
			delete(checkset, value)
		}
	}
	return len(checkset) == 0 //this implies that set is subset of superset
}

// This function adds an elevator ID to the AwareList if it’s not already present.
func AddToAwareList(AwareList []string, id string) []string {
	for i := range AwareList {
		if AwareList[i] == id {
			return AwareList
		}
	}
	return append(AwareList, id)
}
