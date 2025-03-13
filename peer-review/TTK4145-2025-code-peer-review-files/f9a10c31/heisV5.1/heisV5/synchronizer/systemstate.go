package synchronizer

import (
	"fmt"
	"heisV5/config"
	"heisV5/elevator"
	"heisV5/elevio"
	"heisV5/network/peers"
	"reflect"
	"strconv"
)

// **Status for bekreftelse av SystemState**
type ConfirmationStatus int

const (
	Unconfirmed  ConfirmationStatus = iota // 0: Heisen har ikke bekreftet SystemState
	Confirmed                              // 1: Heisen har bekreftet SystemState
	Disconnected                           // 2: Heisen er utilgjengelig
	NotAvailable                           // 3: Heisen er ikke tilgjengelig akkurat nå
)

// **Lokal tilstand for en heis – `LocalState` lagrer status og cab-bestillinger**
type LocalState struct {
	State       elevator.ElevatorState
	CabRequests [config.NumFloors]bool
}

// **SystemState holder all informasjon om heissystemet**
type SystemState struct {
	SeqNum          int
	Origin          int
	ConfirmationMap [config.NumElevators]ConfirmationStatus
	HallRequests    [config.NumFloors][2]bool
	States          [config.NumElevators]LocalState
}

// **Oppdaterer heisens tilstand i SystemState**
func (ss *SystemState) UpdateState(newState elevator.ElevatorState, id int) {
	ss.States[id] = LocalState{
		State:       newState,
		CabRequests: ss.States[id].CabRequests, // Beholder eksisterende cab-bestillinger
	}
}

// **Registrerer en ny bestilling i SystemState (både Hall og Cab)**
func (ss *SystemState) RegisterNewRequest(newOrder elevio.ButtonEvent, id int) {
	if newOrder.Button == elevio.BT_Cab {
		ss.States[id].CabRequests[newOrder.Floor] = true
	} else {
		ss.HallRequests[newOrder.Floor][newOrder.Button] = true
	}

	// Send oppdatert tilstand ut i nettverket
	fmt.Println("[SystemState] New request registered:", newOrder)
}

// **Registrerer kun CabCalls (brukes ved synkronisering når en heis kommer online)**
func (ss *SystemState) RegisterCabRequest(newOrder elevio.ButtonEvent, id int) {
	if newOrder.Button == elevio.BT_Cab {
		ss.States[id].CabRequests[newOrder.Floor] = true
	}
}

// **Fjerner en utført bestilling fra SystemState**
func (ss *SystemState) RemoveRequest(deliveredOrder elevio.ButtonEvent, id int) {
	if deliveredOrder.Button == elevio.BT_Cab {
		ss.States[id].CabRequests[deliveredOrder.Floor] = false
	} else {
		ss.HallRequests[deliveredOrder.Floor][deliveredOrder.Button] = false
	}

	fmt.Println("[SystemState] Request removed:", deliveredOrder)
}

// **Sjekker om alle heiser har bekreftet SystemState**
func (ss *SystemState) FullyConfirmed(id int) bool {
	if ss.ConfirmationMap[id] == Disconnected {
		return false
	}
	for _, status := range ss.ConfirmationMap {
		if status == Unconfirmed {
			return false
		}
	}
	return true
}

// **Sammenligner to SystemState-objekter (ignorerer ConfirmationMap)**
func (oldSs SystemState) Equals(newSs SystemState) bool {
	oldSs.ConfirmationMap = [config.NumElevators]ConfirmationStatus{}
	newSs.ConfirmationMap = [config.NumElevators]ConfirmationStatus{}
	return reflect.DeepEqual(oldSs, newSs)
}

// **Markerer tapte heiser som frakoblet**
func (ss *SystemState) MarkLostPeersAsDisconnected(peers peers.PeerUpdate) {
	for _, peerID := range peers.Lost {
		id, err := strconv.Atoi(peerID)
		if err != nil {
			fmt.Printf("[ERROR] Invalid peer ID format: %s\n", peerID)
			continue
		}

		if id >= 0 && id < len(ss.ConfirmationMap) {
			ss.ConfirmationMap[id] = Disconnected
		} else {
			fmt.Printf("[ERROR] Peer ID out of range: %d\n", id)
		}
	}
}

// **Setter alle andre heiser som Disconnected hvis en heis går offline**
func (ss *SystemState) MarkOthersAsDisconnected(id int) {
	for elev := range ss.ConfirmationMap {
		if elev != id {
			ss.ConfirmationMap[elev] = Disconnected
		}
	}
}

// **Gjenoppretter tilstand for en heis som kommer online igjen**
func (ss *SystemState) RestoreCabRequests(id int) {
	for floor, hasCab := range ss.States[id].CabRequests {
		if hasCab {
			ss.RegisterCabRequest(elevio.ButtonEvent{Floor: floor, Button: elevio.BT_Cab}, id)
		}
	}
	fmt.Println("[SystemState] Restored cab requests for elevator", id)
}

// **Klargjør en ny versjon av SystemState og nullstiller bekreftelser**
func (ss *SystemState) PrepareNewState(id int) {
	ss.SeqNum++
	ss.Origin = id
	for i := range ss.ConfirmationMap {
		if ss.ConfirmationMap[i] == Confirmed {
			ss.ConfirmationMap[i] = Unconfirmed
		}
	}
}
