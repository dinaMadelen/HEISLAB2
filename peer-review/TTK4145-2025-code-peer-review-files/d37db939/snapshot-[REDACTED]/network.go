package main

import (
	"encoding/json"
	"fmt"
	"net"
	"sync"
	"time"
)

// Constants for the network module
const (
	UDP_PORT        = "15657"
	BROADCAST_ADDR  = "255.255.255.255:15657"
	RETRANSMIT_RATE = 500 * time.Millisecond
	TIMEOUT_LIMIT   = 2 * time.Second
	HEARTBEAT_RATE  = 500 * time.Millisecond
)

var (
	peerStatus   sync.Map // peerStatus tracks last update time for each elevator
	stateUpdates = make(chan Elevator)
	udpConn      *net.UDPConn // UDP socket
)

// initNetwork initializes the UDP connection
func initNetwork(elevatorID int, updateChannel chan Elevator) {
	addr, _ := net.ResolveUDPAddr("udp", ":"+UDP_PORT)
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		fmt.Println("Error starting UDP server", err)
		return
	}
	udpConn = conn

	go listenForUpdates() // Listening for messages
	go retransmitState(elevatorID)     // Periodic retransmission of state updates
	go sendHeartbeat(elevatorID)
	go detectFailures()
	go processStateUpdates(updateChannel)
}

// listenForUpdates recives UDP packets and updates PeerStatus
func listenForUpdates() {
	buffer := make([]byte, 1024)
	for {
		n, _, err := udpConn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Println("Error reciving UDP packet", err)
			continue
		}

		var receivedState Elevator

		// check if state recived is valid
		err = json.Unmarshal(buffer[:n], &receivedState)
		if err != nil {
			fmt.Println("Error decoding json:", err)
			continue
		}

		receivedState.lastSeen = time.Now()
		stateUpdates <- receivedState
	}
}

func processStateUpdates(updateChannel chan Elevator) {
	for state := range stateUpdates {
		peerStatus.Store(state.ID, state)
		updateChannel <- state
	}
}

// retransmitState periodically sends the local elevator's state
func retransmitState(elevatorID int) {
	addr, _ := net.ResolveUDPAddr("udp", BROADCAST_ADDR)
	ticker := time.NewTicker(RETRANSMIT_RATE)
	defer ticker.Stop()

	for range ticker.C {
		if state, exists := getPeerStatus(elevatorID); exists {
			sendStateUpdate(state, addr)
		}
	}
}

// sendStateUpdate serializes and broadcasts the state
func sendStateUpdate(elevator Elevator, addr *net.UDPAddr) {
	conn, err := net.Dial("udp", BROADCAST_ADDR)
	if err != nil {
		fmt.Println("Error connecting to broadcast:", err)
		return
	}
	defer conn.Close()

	data, err := json.Marshal(elevator)
	if err != nil {
		fmt.Println("Error with JSON format:", err)
		return
	}

	_, err = udpConn.WriteToUDP(data, addr)
	if err != nil {
		fmt.Println("Error sending UDP packet:", err) // error
	}
}

func sendHeartbeat(elevatorID int) {
	ticker := time.NewTicker(HEARTBEAT_RATE)
	defer ticker.Stop()

	for range ticker.C {
		if state, exist := getPeerStatus(elevatorID); exist {
			state.heartbeat = time.Now()
			peerStatus.Store(elevatorID, state)
		}
	}
}

// detectFailures identifies unresponsive elevators
func detectFailures() {
	for {
		time.Sleep(TIMEOUT_LIMIT)
		now := time.Now()
		peerStatus.Range(func(key, value interface{}) bool {
			id := key.(int)
			state := value.(Elevator)
			if now.Sub(state.heartbeat) > TIMEOUT_LIMIT {
				fmt.Printf("Elevator %d is unresponsive\n", id)
				peerStatus.Delete(id)
			}
			return true
		})
	}
}

func getPeerStatus(id int) (Elevator, bool) {
	val, ok := peerStatus.Load(id)
	if ok {
		return val.(Elevator), true
	}
	return Elevator{}, false
}
