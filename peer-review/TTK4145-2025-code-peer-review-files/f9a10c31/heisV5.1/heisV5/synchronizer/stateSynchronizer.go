package synchronizer

import (
	"fmt"
	"heisV5/config"
	"heisV5/elevator"
	"heisV5/elevio"
	"heisV5/fsm"
	"heisV5/network/peers"
	"strconv"
	"time"
)

type SyncEventType int

const (
	EventNewOrder SyncEventType = iota
	EventOrderCompleted
	EventLocalUpdate
	EventRemoteUpdate
	EventPeerUpdate
)

// SyncEvent wraps any event that affects system state.
type SyncEvent struct {
	Type    SyncEventType
	Payload interface{}
}

// GlobalState aggregates the status of the entire elevator network.
type GlobalState struct {
	Version    int                              // incremented on each local update
	Origin     int                              // ID of the last updater
	PeerActive []bool                           // true if the corresponding elevator is active
	HallCalls  [config.NumFloors][2]bool        // Hall call status (up, down) per floor
	Locals     [config.NumElevators]LocalStatus // Local state of each elevator
}

// LocalStatus holds the state and cab orders for one elevator.
type LocalStatus struct {
	State    elevator.ElevatorState // dynamic state from the FSM
	CabCalls [config.NumFloors]bool // cab call status for that elevator
}

// RunStateSynchronizer reworked into an event-driven design.
// It collects events from several input channels, merges them into a global state,
// and then broadcasts the updated state over the network.
func RunStateSynchronizer(
	confirmedState chan<- GlobalState,
	newOrder <-chan elevio.ButtonEvent,
	orderDone <-chan elevio.ButtonEvent,
	localStateIn <-chan elevator.ElevatorState,
	networkOut chan<- GlobalState,
	networkIn <-chan GlobalState,
	peerUpdates <-chan peers.PeerUpdate,
	selfID int,
) {
	// Initialize our global state.
	var gs GlobalState
	gs.Version = 0
	gs.Origin = selfID
	gs.PeerActive = make([]bool, config.NumElevators)
	for i := range gs.PeerActive {
		gs.PeerActive[i] = (i == selfID)
	}
	// The Locals slice is zero‐initialized.

	// Create a unified channel for all synchronization events.
	events := make(chan SyncEvent, 100)

	// Wrap incoming channels to send events.
	go func() {
		for ev := range newOrder {
			events <- SyncEvent{Type: EventNewOrder, Payload: ev}
		}
	}()
	go func() {
		for done := range orderDone {
			events <- SyncEvent{Type: EventOrderCompleted, Payload: done}
		}
	}()
	go func() {
		for lState := range localStateIn {
			events <- SyncEvent{Type: EventLocalUpdate, Payload: lState}
		}
	}()
	go func() {
		for nState := range networkIn {
			events <- SyncEvent{Type: EventRemoteUpdate, Payload: nState}
		}
	}()
	go func() {
		for pUpd := range peerUpdates {
			events <- SyncEvent{Type: EventPeerUpdate, Payload: pUpd}
		}
	}()

	// Heartbeat ticker for periodic self-updates.
	heartbeat := time.NewTicker(config.HeartbeatTime)
	// Watchdog timer for network disconnects.
	watchdog := time.NewTimer(config.DisconnectTime)

	for {
		select {
		// Process any incoming event.
		case evt := <-events:
			processEvent(evt, &gs, selfID)

		// On heartbeat, update our own local portion and broadcast.
		case <-heartbeat.C:
			current := fsm.GetElevatorState()
			gs.Locals[selfID].State = current
			gs.Version++
			gs.Origin = selfID
			networkOut <- gs

		// If no network activity occurs, mark other peers as inactive.
		case <-watchdog.C:
			disconnectPeers(&gs, selfID)
		}

		// Continuously output the latest state for local confirmation.
		confirmedState <- gs
	}
}

// processEvent merges an incoming event into the global state.
func processEvent(evt SyncEvent, gs *GlobalState, selfID int) {
	switch evt.Type {
	case EventNewOrder:
		ord := evt.Payload.(elevio.ButtonEvent)
		// For cab calls, update our own cab calls.
		if ord.Button == elevio.BT_Cab {
			gs.Locals[selfID].CabCalls[ord.Floor] = true
		} else {
			// For hall calls, update the corresponding floor and button.
			if ord.Button == elevio.BT_HallUp {
				gs.HallCalls[ord.Floor][0] = true
			} else if ord.Button == elevio.BT_HallDown {
				gs.HallCalls[ord.Floor][1] = true
			}
		}
		// (Optionally log the new order.)
		fmt.Printf("[Sync] Registered new order: %+v\n", ord)

	case EventOrderCompleted:
		done := evt.Payload.(elevio.ButtonEvent)
		if done.Button == elevio.BT_Cab {
			gs.Locals[selfID].CabCalls[done.Floor] = false
		} else {
			if done.Button == elevio.BT_HallUp {
				gs.HallCalls[done.Floor][0] = false
			} else if done.Button == elevio.BT_HallDown {
				gs.HallCalls[done.Floor][1] = false
			}
		}
		fmt.Printf("[Sync] Cleared order: %+v\n", done)

	case EventLocalUpdate:
		// Update our own elevator state.
		lState := evt.Payload.(elevator.ElevatorState)
		gs.Locals[selfID].State = lState

	case EventRemoteUpdate:
		remote := evt.Payload.(GlobalState)
		// Use a simple “latest version wins” rule.
		if remote.Version > gs.Version || (remote.Version == gs.Version && remote.Origin > gs.Origin) {
			*gs = remote
			// Ensure that our own peer status remains active.
			gs.PeerActive[selfID] = true
			fmt.Printf("[Sync] Merged remote state (v%d from %d)\n", remote.Version, remote.Origin)
		}

	case EventPeerUpdate:
		peerUpd := evt.Payload.(peers.PeerUpdate)
		// Mark lost peers as inactive.
		for _, lost := range peerUpd.Lost {
			if id, err := strconv.Atoi(lost); err == nil && id >= 0 && id < len(gs.PeerActive) {
				gs.PeerActive[id] = false
			}
		}
		// Mark present peers as active.
		for _, peer := range peerUpd.Peers {
			if id, err := strconv.Atoi(peer); err == nil && id >= 0 && id < len(gs.PeerActive) {
				gs.PeerActive[id] = true
			}
		}
	}
}

// disconnectPeers marks all peers except self as inactive.
func disconnectPeers(gs *GlobalState, selfID int) {
	for i := range gs.PeerActive {
		if i != selfID {
			gs.PeerActive[i] = false
		}
	}
	fmt.Println("[Sync] Watchdog timeout: marked all other peers as disconnected")
}
