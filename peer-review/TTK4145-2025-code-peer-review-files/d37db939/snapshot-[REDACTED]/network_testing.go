package main

import (
	"encoding/json"
	"net"
	"testing"
	"time"
)

func TestNetworkSendReceive(t *testing.T) {
	// Simulate an elevator sending a state update
	elevatorID := 1
	testState := Elevator{
		ID:        elevatorID,
		floor:     2,
		dirn:      DIRN_UP,
		active:    true,
		state: ELEVSTATE_MOVING,
	}

	// Mock UDP server
	addr, _ := net.ResolveUDPAddr("udp", ":"+UDP_PORT)
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		t.Fatalf("Failed to start mock UDP server: %v", err)
	}
	defer conn.Close()

	// Start a goroutine to listen for state updates
	go func() {
		buffer := make([]byte, 1024)
		n, _, _ := conn.ReadFromUDP(buffer)
		var receivedState Elevator
		json.Unmarshal(buffer[:n], &receivedState)

		// Store in sync.Map
		peerStatus.Store(receivedState.ID, receivedState)
	}()

	// Send a test state update
	sendStateUpdate(testState, addr)

	// Wait for message processing
	time.Sleep(500 * time.Millisecond)

	// Verify that the state was stored
	storedState, exists := getPeerStatus(elevatorID)
	if !exists {
		t.Errorf("State was not stored in sync.Map")
	}

	if storedState.floor != 2 || storedState.dirn != DIRN_UP || !storedState.active {
		t.Errorf("Stored state does not match expected values")
	}
}

func TestConcurrentElevatorUpdates(t *testing.T) {
	elevatorCount := 5

	// Simulate multiple elevators sending state updates
	for i := 1; i <= elevatorCount; i++ {
		go func(id int) {
			state := Elevator{
				ID:        id,
				floor:     id, // Different floor for each
				dirn:      DIRN_STOP,
				active:    true,
				state: ELEVSTATE_IDLE,
			}
			peerStatus.Store(id, state)
		}(i)
	}

	// Wait for all updates
	time.Sleep(500 * time.Millisecond)

	// Verify all states were stored correctly
	for i := 1; i <= elevatorCount; i++ {
		storedState, exists := getPeerStatus(i)
		if !exists || storedState.floor != i {
			t.Errorf("Elevator %d state not stored correctly", i)
		}
	}
}

func TestFailureDetection(t *testing.T) {
	elevatorID := 3
	state := Elevator{
		ID:        elevatorID,
		floor:     2,
		active:    true,
		heartbeat: time.Now().Add(-3 * TIMEOUT_LIMIT), // Simulate expired heartbeat
	}

	peerStatus.Store(elevatorID, state)

	// Run failure detection once
	detectFailures()

	// Verify that the elevator was removed
	_, exists := getPeerStatus(elevatorID)
	if exists {
		t.Errorf("Elevator %d was not removed after timeout", elevatorID)
	}
}
