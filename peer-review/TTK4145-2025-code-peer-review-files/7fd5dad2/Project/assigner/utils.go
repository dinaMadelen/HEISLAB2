package assigner

import (
	. "Project/dataenums"
)

func initPayloadToNetwork() FromAssignerToNetwork {
	payload := FromAssignerToNetwork{
		HallRequests: [NFloors][NButtons]ButtonState{},
		States:       make(map[string]HRAElevState),
	}
	return payload
}
func updateLightStates(payload FromNetworkToAssigner, myID int) [NFloors][NButtons]ButtonState {
	var updatedLights [NFloors][NButtons]ButtonState
	updatedLights = payload.HallOrderList[myID]
	for floor := 0; floor < NFloors; floor++ {
		if payload.ElevatorList[myID].CabRequests[floor] {
			updatedLights[floor][BCab] = OrderAssigned
		}
	}

	return updatedLights
}

func handlePayloadFromNetwork(
	payload FromAssignerToNetwork,
	netPayload FromNetworkToAssigner,
	nodeID int,
) FromAssignerToNetwork {
	for f := 0; f < NFloors; f++ {
		for b := 0; b < NButtons; b++ {
			incomingState := netPayload.HallOrderList[nodeID][f][b]
			localState := payload.HallRequests[f][b]
			if localState == OrderComplete && incomingState == OrderAssigned {
				// do nothing; stay OrderComplete.
			} else {
				payload.HallRequests[f][b] = incomingState
			}

		}
	}
	return payload
}

func handlePayloadFromElevator(fromElevator FromDriverToAssigner,
	toNetwork FromAssignerToNetwork, nodeID string) FromAssignerToNetwork {

	behavior, direction, cabRequests := convertElevatorState(fromElevator.Elevator)
	toNetwork.States[nodeID] = HRAElevState{
		Behavior:    behavior,
		Floor:       fromElevator.Elevator.CurrentFloor,
		Direction:   direction,
		CabRequests: cabRequests,
	}
	toNetwork.ActiveSatus = fromElevator.Elevator.ActiveSatus
	toNetwork = orderComplete(toNetwork, nodeID, fromElevator.CompletedOrders)

	return toNetwork
}

func convertElevatorState(e Elevator) (string, string, []bool) {
	behavior := ebToString(e.CurrentBehaviour)
	direction := elevDirToString(e.Dirn)
	cabRequests := make([]bool, NFloors)

	for f := 0; f < NFloors; f++ {
		cabRequests[f] = e.Requests[f][BCab]
	}
	return behavior, direction, cabRequests
}

func ebToString(behaviour ElevatorBehaviour) string {
	switch behaviour {
	case EBIdle:
		return "idle"
	case EBDoorOpen:
		return "doorOpen"
	case EBMoving:
		return "moving"
	default:
		return "Unknown"
	}
}
func elevDirToString(d HWMotorDirection) string {
	switch d {
	case MDDown:
		return "down"
	case MDStop:
		return "stop"
	case MDUp:
		return "up"
	default:
		return "DirUnknown"
	}
}
