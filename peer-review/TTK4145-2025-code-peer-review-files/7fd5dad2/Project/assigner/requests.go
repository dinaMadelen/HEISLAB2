package assigner

import (
	. "Project/dataenums"
)

func buttonPressed(payload FromAssignerToNetwork, ElevatorName string,
	btnEvent ButtonEvent) FromAssignerToNetwork {
	switch btnEvent.Button {
	case BHallUp:
		if payload.HallRequests[btnEvent.Floor][BHallUp] != OrderComplete {
			payload.HallRequests[btnEvent.Floor][BHallUp] = ButtonPressed
		}
	case BHallDown:
		if payload.HallRequests[btnEvent.Floor][BHallDown] != OrderComplete {
			payload.HallRequests[btnEvent.Floor][BHallDown] = ButtonPressed
		}

	case BCab:
		payload.States[ElevatorName].CabRequests[btnEvent.Floor] = true

	}
	return payload
}

func orderComplete(payload FromAssignerToNetwork, elevatorName string,
	completedOrders [NFloors][NButtons]bool) FromAssignerToNetwork {
	for floor := 0; floor < NFloors; floor++ {
		for btn := BHallUp; btn <= BCab; btn++ {
			if completedOrders[floor][btn] {
				switch btn {
				case BHallUp:
					payload.HallRequests[floor][BHallUp] = OrderComplete
				case BHallDown:
					payload.HallRequests[floor][BHallDown] = OrderComplete
				case BCab:
					payload.States[elevatorName].CabRequests[floor] = false
				}
			}
		}
	}
	return payload
}
