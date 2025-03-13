package fsm

import (
	"fmt"
	"heisV5/elevator"
	"heisV5/elevio"
	"heisV5/requests"
	"time"
)

var assignedOrdersC chan [4][3]bool // Bestillinger fra hallassigner
var stateUpdateC chan elevator.ElevatorState

// **Init-funksjon for å sette opp kanaler**
func InitFSM(stateChan chan elevator.ElevatorState, ordersChan chan [4][3]bool) {
	stateUpdateC = stateChan
	assignedOrdersC = ordersChan
	go RunFSM()
}

// **Global ElevatorState**
var elevatorState = elevator.ElevatorState{
	IsObstructed:    false,
	Motorstop:       false,
	Activity:        elevator.Idle,
	CurrentFloor:    -1,
	MovingDirection: elevio.MD_Stop,
}

// **Lytter på oppdaterte bestillinger fra hallassigner**
func RunFSM() {
	for {
		select {
		case assignedOrders := <-assignedOrdersC:
			fmt.Println("[FSM] Received updated RequestMatrix")
			handleUpdatedOrders(assignedOrders)
		}
	}
}

// **Håndterer oppdaterte bestillinger fra hallassigner**
func handleUpdatedOrders(assignedOrders [4][3]bool) {
	fmt.Println("[FSM] Processing new order matrix")

	// Velger retning basert på de nye bestillingene
	elevatorState.MovingDirection = requests.ChooseDirection(elevatorState.CurrentFloor, elevatorState.MovingDirection)

	if elevatorState.MovingDirection == elevio.MD_Stop {
		elevatorState.Activity = elevator.Idle
	} else {
		elevatorState.Activity = elevator.Moving
	}

	elevio.SetDoorOpenLamp(false)
	elevio.SetMotorDirection(elevatorState.MovingDirection)

	// Send oppdatert tilstand til synkroniseringssystemet
	stateUpdateC <- elevatorState
}

// **Heisen ankommer en etasje**
func OnFloorArrival(floor int) {
	elevatorState.CurrentFloor = floor
	elevio.SetFloorIndicator(floor)

	switch elevatorState.Activity {
	case elevator.Moving:
		if requests.ShouldStopAtFloor(floor, elevatorState.MovingDirection) {
			transitionToDoorOpen()
		}
	case elevator.DoorOpen:
		fmt.Println("[FSM] Door already open, ignoring floor event")
	case elevator.Idle:
		fmt.Println("[FSM] Elevator detected floor signal while idle")
	}
}

// **Håndterer obstruksjon**
func OnObstructionChange(obstructed bool) {
	elevatorState.IsObstructed = obstructed

	if elevatorState.IsObstructed {
		fmt.Println("[FSM] Obstruction detected -> Keeping door open")
		elevio.SetDoorOpenLamp(true)
	} else {
		fmt.Println("[FSM] Obstruction cleared -> Normal operation resuming")
		go func() {
			time.Sleep(3 * time.Second)
			handleUpdatedOrders([4][3]bool{}) // Trigger ny bestillingshåndtering
		}()
	}

	// Send oppdatert tilstand til synkroniseringssystemet
	stateUpdateC <- elevatorState
}

// **Åpne døren og vent**
func transitionToDoorOpen() {
	fmt.Println("[FSM] Door opens at floor", elevatorState.CurrentFloor)
	elevatorState.Activity = elevator.DoorOpen
	elevio.SetMotorDirection(elevio.MD_Stop)
	elevio.SetDoorOpenLamp(true)

	// Send oppdatert state til synkroniseringssystemet
	stateUpdateC <- elevatorState

	go func() {
		time.Sleep(3 * time.Second)
		elevio.SetDoorOpenLamp(false)
		handleUpdatedOrders([4][3]bool{}) // Starter ny sjekk for bestillinger
	}()
}

// **Returnerer nåværende tilstand til SystemState**
func GetElevatorState() elevator.ElevatorState {
	return elevatorState
}
