package connectivity

import (
	"fmt"
	"log"
	"sync"
)

// This file contains the status of which elevators are are online and ofline based on this elevators view

var (
	// isOnline is a list of online elevators
	// The index of the list is the elevator ID
	// The value is true if the elevator is online, false if it is offline
	// Online or offline is based on if we receve message from it or not
	isOnline       = [NR_OF_ELEVATORS]bool{}
	isOnline_mutex sync.Mutex
)

func init() {
	// Initialize the isOnline list
	isOnline[ID] = true
}

func get_isOnline() [NR_OF_ELEVATORS]bool {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	return isOnline
}

func set_isOnline(id int, state bool) {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	isOnline[id] = state
}

// AddElevatorOnline sets the elevator ID to online in the isOnline list
func SetElevatorOnline(elevatorID int) {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()

	//If a valid id
	if elevatorID >= 0 && elevatorID < NR_OF_ELEVATORS {

		// If is only to make the print only appare if there is a chainge in state
		if isOnline[elevatorID] {
			fmt.Println("Setting ElevatorID:", elevatorID, "to ONLINE!")
		}

		isOnline[elevatorID] = true

	} else {
		log.Fatal("Not valid elevatorID when SetElevatorOnline()", elevatorID)
	}
}

// RemoveElevatorOnline sets the elevator ID to offline in the isOnline list
func SetElevatorOffline(elevatorID int) {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	if elevatorID >= 0 && elevatorID < len(isOnline) {

		// If is only to make the print only appare if there is a chainge in state
		if isOnline[elevatorID] {
			fmt.Println("Setting ElevatorID:", elevatorID, "to OFLINE!")
		}

		isOnline[elevatorID] = false
	} else {
		log.Fatal("Not valid elevatorID when SetElevatorOffline():", elevatorID)
	}
}

// Return true if is online
func IsOnline(elevatorID int) bool {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	if elevatorID >= 0 && elevatorID < len(isOnline) {
		return isOnline[elevatorID]
	}

	log.Fatal("Not valid elevatorID when IsOnline()", elevatorID)
	return false
}

// PrintIsOnline prints the status of all elevators
func PrintIsOnline() {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	for i, online := range isOnline {
		status := "offline"
		if online {
			status = "online"
		}
		log.Printf("Elevator %d is %s\n", i, status)
	}
}

func Self_only_online() bool {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()
	for i := 0; i < NR_OF_ELEVATORS; i++ {
		if i != ID {
			if isOnline[i] {
				return false
			}
		}
	}
	//Self is only online
	return true
}

func Get_all_online_ids() []int {
	isOnline_mutex.Lock()
	defer isOnline_mutex.Unlock()

	var online_elevators []int
	for i, online := range isOnline {
		if online {
			online_elevators = append(online_elevators, i)
		}
	}
	return online_elevators
}

func get_lowest_online_id() int {
	//returning the online elevator with lowest id
	return Get_all_online_ids()[0]
}
