package elevator

import (
	"heisV5/config"
	"heisV5/elevio"
)

type Activity int

const (
	Idle Activity = iota
	Moving
	DoorOpen
)

func (b Activity) ToString() string {
	switch b {
	case Idle:
		return "Idle"
	case Moving:
		return "Moving"
	case DoorOpen:
		return "DoorOpen"
	default:
		return "Unknown"
	}
}

// **Elevator-struktur som lagrer tilstand for en heis**
type ElevatorState struct {
	CurrentFloor    int
	Direction       elevio.MotorDirection
	RequestMatrix   [config.NumFloors][config.NumButtons]bool // Oppdateres fra SystemState via assigner.go
	Motorstop       bool
	IsObstructed    bool
	Activity        Activity
	MovingDirection elevio.MotorDirection
}

// **Oppretter en ny heis med standardverdier**
func NewElevator() ElevatorState {
	return ElevatorState{
		CurrentFloor: -1, // Ukjent startposisjon
		Direction:    elevio.MD_Stop,
	}
}

// **Oppdaterer RequestMatrix fra SystemState via assigner.go**
func (e *ElevatorState) UpdateRequestMatrix(updatedMatrix [4][3]bool) {
	e.RequestMatrix = updatedMatrix
}

// **Sender CabCalls via kanal i stedet for å lagre lokalt**
func (e *ElevatorState) AddCabRequest(floor int, cabRequestC chan<- elevio.ButtonEvent) {
	cabRequestC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_Cab}
}

// **Fjerner bestillinger i en etasje**
func (e *ElevatorState) ClearRequestsAtFloor(floor int) {
	for btn := 0; btn < 3; btn++ {
		e.RequestMatrix[floor][btn] = false
	}
}

// **Sjekker om heisen skal stoppe i en etasje**
func (e *ElevatorState) ShouldStopAtFloor(floor int) bool {
	return e.RequestMatrix[floor][elevio.BT_Cab] ||
		(e.Direction == elevio.MD_Up && e.RequestMatrix[floor][elevio.BT_HallUp]) ||
		(e.Direction == elevio.MD_Down && e.RequestMatrix[floor][elevio.BT_HallDown])
}

// **Gjenoppretter CabCalls via kanal hvis heisen går offline og kommer tilbake**
func (e *ElevatorState) RestoreCabRequests(cabRequestC chan<- elevio.ButtonEvent) {
	for floor := 0; floor < len(e.RequestMatrix); floor++ {
		if e.RequestMatrix[floor][elevio.BT_Cab] {
			cabRequestC <- elevio.ButtonEvent{Floor: floor, Button: elevio.BT_Cab}
		}
	}
}
