package driver

import (
	. "Project/dataenums"
	"fmt"
)

func chooseDirection(elevator Elevator) Elevator {
	dirnBehaviour := decideDirection(elevator)
	elevator.Dirn = dirnBehaviour.Dirn
	elevator.CurrentBehaviour = dirnBehaviour.Behaviour
	return elevator
}

func decideDirection(elevator Elevator) DirnBehaviourPair {
	switch elevator.Dirn {
	case MDUp:
		return decideDirectionUp(elevator)
	case MDDown:
		return decideDirectionDown(elevator)
	case MDStop:
		return decideDirectionStop(elevator)
	default:
		return DirnBehaviourPair{MDStop, EBIdle}
	}
}

func decideDirectionUp(elevator Elevator) DirnBehaviourPair {
	switch {
	case requestsAbove(elevator):
		return DirnBehaviourPair{MDUp, EBMoving}
	case requestsHere(elevator):
		return DirnBehaviourPair{MDStop, EBIdle} 
	case requestsBelow(elevator):
		return DirnBehaviourPair{MDDown, EBMoving}
	default:
		return DirnBehaviourPair{MDStop, EBIdle}
	}
}

func decideDirectionDown(elevator Elevator) DirnBehaviourPair {
	switch {
	case requestsBelow(elevator):
		return DirnBehaviourPair{MDDown, EBMoving}
	case requestsHere(elevator):
		return DirnBehaviourPair{MDStop, EBIdle} 
	case requestsAbove(elevator):
		return DirnBehaviourPair{MDUp, EBMoving}
	default:
		return DirnBehaviourPair{MDStop, EBIdle}
	}

}

func decideDirectionStop(elevator Elevator) DirnBehaviourPair {
	switch {
	case requestsHere(elevator):
		return DirnBehaviourPair{MDStop, EBIdle}
	case requestsAbove(elevator):
		return DirnBehaviourPair{MDUp, EBMoving}
	case requestsBelow(elevator):
		return DirnBehaviourPair{MDDown, EBMoving}
	default:
		return DirnBehaviourPair{MDStop, EBIdle}
	}
}

func requestsAbove(elevator Elevator) bool {
	for f := elevator.CurrentFloor + 1; f < NFloors; f++ {
		for btn := BHallUp; btn <= BCab; btn++ {
			if elevator.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func requestsBelow(elevator Elevator) bool {
	for f := 0; f < elevator.CurrentFloor; f++ {
		for btn := BHallUp; btn <= BCab; btn++ {
			if elevator.Requests[f][btn] {
				return true
			}
		}
	}
	return false
}

func requestsHere(elevator Elevator) bool {
	for btn := BHallUp; btn <= BCab; btn++ {
		if elevator.Requests[elevator.CurrentFloor][btn] {
			return true
		}
	}
	return false
}

// TODO REMOVE
func ElevatorPrint(e Elevator) {
	fmt.Println("\n  +--------------------+")
	fmt.Printf(
		"  |floor = %-2d          |\n"+
			"  |dirn  = %-12s|\n"+
			"  |behav = %-12s|\n",
		e.CurrentFloor,
		ElevDirToString(e.Dirn),
		EBToString(e.CurrentBehaviour),
	)
	fmt.Println("  +--------------------+")
	fmt.Println("  |  | up  | dn  | cab |")
	for f := NFloors - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := BHallUp; btn <= BCab; btn++ {
			if (f == NFloors-1 && btn == BHallUp) ||
				(f == 0 && btn == BHallDown) {
				fmt.Print("|     ")
			} else {
				if e.Requests[f][btn] {
					fmt.Print("|  #  ")
				} else {
					fmt.Print("|  -  ")
				}
			}
		}
		fmt.Println("|")
	}
	fmt.Println("  +--------------------+")
}

func EBToString(behaviour ElevatorBehaviour) string {
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
func ElevDirToString(d HWMotorDirection) string {
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
