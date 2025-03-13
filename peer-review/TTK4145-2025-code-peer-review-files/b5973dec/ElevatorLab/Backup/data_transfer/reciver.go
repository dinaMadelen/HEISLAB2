// coordinator/tcpserver.go
package data_transfer

import (
	"encoding/json"
	"fmt"
	"net"
)

// Elevator state struct
const NumFloors int = 4
const NumButtons int = 3

type State struct {
	Elevator_id        int
	Elevator_floor     int
	Elevator_dir       int
	Elevator_behaviour int
	Elevator_request   [NumFloors][NumButtons]int
}

var ListOfStates [3]State

func ReciveElevatorState(port string) {
	listener, err := net.Listen("tcp", "localhost:"+port)
	if err != nil {
		fmt.Println("Failed to start server:", err)
		return
	}
	defer listener.Close()
	fmt.Println("Server listening on port", port)

	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Failed to accept connection:", err)
			continue
		}

		go handleConnection(conn)
	}
}

// handleConnection decodes incoming elevator state and processes it
func handleConnection(conn net.Conn) {
	defer conn.Close()

	decoder := json.NewDecoder(conn)
	var state State

	if err := decoder.Decode(&state); err != nil {
		fmt.Println("Failed to decode state:", err)

	}
	fmt.Printf("Received state update: \n")

	ListOfStates[state.Elevator_id-1] = state
}

func ReciveHeartBeat(port string, ip string) {
	listener, err := net.Listen("tcp", ip+":"+port)
	if err != nil {
		fmt.Println("Failed to start server:", err)
		return
	}
	defer listener.Close()
	fmt.Println("Server listening on port", port)

	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Failed to accept connection:", err)
			continue
		}

		go handleHeartBeat(conn)
	}
}

func handleHeartBeat(conn net.Conn) {
	var beat string
	defer conn.Close()

	decoder := json.NewDecoder(conn)

	if err := decoder.Decode(&beat); err != nil {
		fmt.Println("Failed to decode heartbeat:", err)

	}
	fmt.Println(beat)

}
