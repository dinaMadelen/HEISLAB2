package commonstate

import (
	"Driver-go/config"
	"Driver-go/elevator"
	"Driver-go/elevio"
	"Driver-go/network/peers"
	"reflect"
	"strconv"
)

type Acknowledgement int

const (
	Acked Acknowledgement = iota
	NotAcked
	NotAvailable
)

// the state of a single elevator and its cabcalls
type LocalState struct {
	State     elevator.State
	CabOrders [config.NumFloors]bool
}

// To keep track of the state of the whole system (all elevators and orders),
// as well as making sure all elevators are syncronized (have the same worldview)
type CommonState struct {
	ElevatorStates [config.NumElevators]LocalState
	HallOrders     [config.NumFloors][2]bool            // request from floors, up or down direction
	Origin         int                                  // which elevator is sending the common state
	SeqNumber      int                                  // icrements with every new common state
	AckMap         [config.NumElevators]Acknowledgement // to keep track of the other elevators acknowledgement, and make
	// sure that all the elevators have received the latest state
}

func (common_state *CommonState) addOrder(id int, newOrder elevio.ButtonEvent) {
	if newOrder.Button == elevio.BT_Cab {
		common_state.ElevatorStates[id].CabOrders[newOrder.Floor] = true
	} else {
		common_state.HallOrders[newOrder.Floor][int(newOrder.Button)] = true
	}
}

func (common_state *CommonState) removeOrder(id int, finishedOrder elevio.ButtonEvent) {
	if finishedOrder.Button == elevio.BT_Cab {
		common_state.ElevatorStates[id].CabOrders[finishedOrder.Floor] = false
	} else {
		common_state.HallOrders[finishedOrder.Floor][int(finishedOrder.Button)] = false
	}
}

// for if the elevator is disconnected from the network
func (common_state *CommonState) makeOthersUnavailable(id int) {
	for i := 0; i < config.NumElevators; i++ {
		if i != id {
			common_state.AckMap[i] = NotAvailable
		}
	}
}

// to remove disconnected elevators from the commonstate, so that the others can still be fully acknowledged
func (common_state *CommonState) makeLostPeersUnavailable(peers peers.PeerUpdate) {
	for _, lostID := range peers.Lost {
		intLostID, error := strconv.Atoi(lostID)
		if error == nil {
			common_state.AckMap[intLostID] = NotAvailable
		}
	}
}

// syncronized (all elevators have the same worldview) if the common state is fully acknowledged
func (common_state *CommonState) fullyAcknowledged(id int) bool {
	if common_state.AckMap[id] == NotAvailable {
		return false
	}
	for index := range common_state.AckMap {
		if common_state.AckMap[index] == NotAcked {
			return false
		}
	}
	return true
}

// checks if two common states are equal with exeption of the AckMap
func (common_state *CommonState) equalCheck(otherCS CommonState) bool {
	common_state.AckMap = [config.NumElevators]Acknowledgement{}
	otherCS.AckMap = [config.NumElevators]Acknowledgement{}
	return reflect.DeepEqual(common_state, otherCS)
}

func (common_state *CommonState) updateElevatorState(id int, newState elevator.State) {
	common_state.ElevatorStates[id].State = newState
}

// prepares a new common state to be sent
func (common_state *CommonState) prepareNewCS(id int) {
	common_state.Origin = id
	common_state.SeqNumber++
	for i := 0; i < config.NumElevators; i++ {
		if i == id {
			common_state.AckMap[i] = Acked
		} else {
			common_state.AckMap[i] = NotAcked
		}
	}
}

/*
We must still acutally implement a state machine to use/update the common state according to
the logic of how we want the elevators to behave.
*/
