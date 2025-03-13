package elevator_logic

import (
	"elevator"
	"encoding/json"
	"fmt"
	"hra"
	"net"
	"sync"
	"time"
	"comm"
	"strconv"
)

var allElevStates = make(map[int]elevator.ElevStates)
var globalHallRequest [][2]bool
var activeElevatorConnections = make(map[int]net.Conn)
var mutex sync.Mutex


func MasterLogic_StartMasterServer(port string) {
	ln, err := net.Listen("tcp" , port)
	if err != nil {
		fmt.Println("Error starting master: ", err)
		return
	}
	defer ln.Close()

	fmt.Println("Master listening on", port)

	for {
		conn, err := ln.Accept()
		if err != nil {
			fmt.Println("Error accepting connection: ", err)
			continue
		}
		go MasterLogic_handleElevatorConnection(conn)
	}
}



func MasterLogic_handleElevatorConnection(conn net.Conn) {
	defer conn.Close()
	decoder := json.NewDecoder(conn)

	for {
		var state elevator.ElevStates
		err := decoder.Decode(&state)
		if err != nil {
			fmt.Println("Error decoding elevator state: ", err)
			return
		}
		mutex.Lock()
		allElevStates[state.ID] = state
		mutex.Unlock()

		fmt.Printf("Recieved state from elevator %d: %+v\n", state.ID, state)
	}
}


// Not finished with this, we are struggeling
func MasterLogic_runHRAUpdater() {
	for {
		time.Sleep(1*time.Second)

		mutex.Lock()
		hallAssignments := hra.SendStateToHRA(allElevStates, globalHallRequest)
		mutex.Unlock()

		if hallAssignments != nil {
			fmt.Println("New hall assignments: ", hallAssignments)

			//sending back to elevators
			for id, hallrequest := range *hallAssignments {
				elevatorID, err := strconv.Atoi(id)
				if err != nil {
					fmt.Println("Error parsing elevator ID: ", err)
					continue
				}

				conn, exists := activeElevatorConnections[elevatorID]
				if !exists {
					fmt.Printf("No connection found for elevator %d\n", elevatorID)
					continue
				}

				comm.Comm_sendMessage(hallrequest, conn)
			}
		}
	}
}

func MasterLogic_sendUpdatedRequestToElevators(output *map[string][][2]bool, connections map[int]net.Conn) {

}