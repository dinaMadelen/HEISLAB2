package fsm

import (
	"ElevatorProject/elevio"
	"ElevatorProject/network/node"
	"ElevatorProject/orderDistribution"
	"ElevatorProject/timer"
	"fmt"
	"time"
)

var (
	e orderDistribution.ElevatorState
	//outputDevice ElevOutputDevice
)

var (
	s orderDistribution.State
)

var (
	n node.NodeUpdate
)

var elevatorStates orderDistribution.ElevatorStates

func setAllLights(e *orderDistribution.ElevatorState) {
	for floor := 0; floor < elevio.NumFloors; floor++ {
		for btn := 0; btn < elevio.NumButtonTypes; btn++ {
			state := e.Requests[floor][btn]
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, state)
		}
	}
}

func FsmInit(id string) {

	e = orderDistribution.NewElevatorState(elevio.NumFloors)

	if initialFloor := elevio.GetFloor(); initialFloor == -1 {
		elevio.SetMotorDirection(elevio.MD_Down)
		//e.Dirn = orderDistribution.DirnDown
		e.Behaviour = orderDistribution.ElevatorBehaviour(1)
	} else {
		e.Dirn = orderDistribution.DirnStop
		e.Behaviour = orderDistribution.ElevatorBehaviour(0)
	}
	elevatorStates := make(orderDistribution.ElevatorStates)
	s = orderDistribution.State{
		ID:    id,
		State: e,
		Time:  0,
	}
	elevatorStates[s.ID] = s.State

	fmt.Println(elevatorStates)
}

func FsmOnRequestButtonPress(btnFloor int, btnType elevio.ButtonType) {
	e.Requests[btnFloor][btnType] = true

	hallRequests := make([][]bool, len(e.Requests))

	if n.Master == s.ID {
		for i := range e.Requests {
			if len(e.Requests[i]) >= 2 {
				hallRequests[i] = e.Requests[i][:2] // Ta kun de to første kolonnene
			} else {
				hallRequests[i] = append([]bool{}, e.Requests[i]...) // Kopier alt hvis mindre enn 2 kolonner
			}
		}
		orderDistribution.OptimalHallRequests(hallRequests, elevatorStates)
	}

	if e.Behaviour == orderDistribution.DoorOpen {
		if e.Floor == btnFloor {
			// Restart timer hvis heisen allerede er på riktig etasje
			timer.TimerStart(float64(3*time.Second) / float64(time.Second))

			orderDistribution.ClearReqsAtFloor(&e, nil)
		}
	} else if e.Behaviour == orderDistribution.Idle {
		// Start bevegelse umiddelbart
		e.Dirn = orderDistribution.ChooseDirection(&e)
		if e.Dirn != orderDistribution.DirnStop {
			elevio.SetMotorDirection(elevio.MotorDirection(e.Dirn))
			e.Behaviour = orderDistribution.Moving
		} else {
			// Ingen bestillinger, gå til idle med dørene åpne
			e.Behaviour = orderDistribution.DoorOpen
			elevio.SetDoorOpenLamp(true)
			timer.TimerStart(float64(3*time.Second) / float64(time.Second))
		}
	}
	setAllLights(&e)
}

func FsmOnFloorArrival(newFloor int) {
	// Oppdater heisens nåværende etasje
	e.Floor = newFloor

	// Oppdater etasjeindikatoren
	elevio.SetFloorIndicator(e.Floor)

	hallRequests := make([][]bool, len(e.Requests))

	if n.Master == s.ID {
		for i := range e.Requests {
			if len(e.Requests[i]) >= 2 {
				hallRequests[i] = e.Requests[i][:2] // Ta kun de to første kolonnene
			} else {
				hallRequests[i] = append([]bool{}, e.Requests[i]...) // Kopier alt hvis mindre enn 2 kolonner
			}
		}
		orderDistribution.OptimalHallRequests(hallRequests, elevatorStates)
	}

	switch e.Behaviour {
	case orderDistribution.ElevatorBehaviour(1):
		// Sjekk om heisen skal stoppe i denne etasjen
		if orderDistribution.ShouldStop(&e) {
			// Stopp motoren
			elevio.SetMotorDirection(elevio.MD_Stop)

			// Slå på dørlyset
			elevio.SetDoorOpenLamp(true)

			// Rydd forespørsler for nåværende etasje
			orderDistribution.ClearReqsAtFloor(&e, nil)

			// Start timer for å holde dørene åpne
			timer.TimerStart(3.0)

			// Oppdater knappelysene
			setAllLights(&e)

			// Endre heisens oppførsel til "DoorOpen"
			e.Behaviour = orderDistribution.ElevatorBehaviour(2)
		}

	default:
		// Ingen spesifikk handling for andre oppførsler
	}
}

func FsmOnDoorTimeout() {
	if e.Behaviour == orderDistribution.DoorOpen {
		// Velger ny retning basert på bestillinger
		e.Dirn = orderDistribution.ChooseDirection(&e)

		if e.Dirn != orderDistribution.DirnStop {
			// Lukk dørene og start motor
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevio.MotorDirection(e.Dirn))
			e.Behaviour = orderDistribution.Moving
		} else {
			// Ingen flere bestillinger, gå til Idle
			elevio.SetDoorOpenLamp(false)
			e.Behaviour = orderDistribution.Idle
		}
	}
}
