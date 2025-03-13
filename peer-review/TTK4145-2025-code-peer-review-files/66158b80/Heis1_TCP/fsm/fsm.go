package fsm

import (
	"Driver-go/elevio"
	"fmt"
	"sync"
)

var (
	elevator       Elevator = NewElevator()
	elevator_mutex sync.Mutex
)

func GetElevatorStruct() Elevator {
	elevator_mutex.Lock()
	defer elevator_mutex.Unlock()
	return elevator
}

//ElevatorOUtputDevice er den utdelte go driverern!

func setAllLights(elevator Elevator) {
	for floor := 0; floor < NumFloors; floor++ {
		for btn := 0; btn < NumButtons; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, elevator.Requests[floor][btn])
		}
	}
}

// func fsm_onInitBetweenFloors() {
// 	elevio.SetMotorDirection(elevio.MD_Down)
// 	elevator.dirn = elevio.MD_Down
// 	elevator.behaviour = EB_Moving
// }

func Fsm_onRequestButtonPress(btn_floor int, btn_type elevio.ButtonType) {
	fmt.Printf("\n\nfsm_onRequestButtonPress(%d)\n", btn_floor)

	switch elevator.Behaviour {
	case EB_DoorOpen:
		if requests_shouldClearImmediately(elevator, btn_floor, btn_type) { // Hvis heisen allerede er i etasjen og knappen er trykket inn
			TimerStart(elevator.DoorOpenDuration_s) // Start dørtimeren på nytt
		} else {
			elevator.Requests[btn_floor][btn_type] = true // Ellers legg forespørselen til i køen
		}
	case EB_Moving:
		elevator.Requests[btn_floor][btn_type] = true // I bevegelse, så ikke gjøre annet enn å legge til i køen

	case EB_Idle:
		elevator.Requests[btn_floor][btn_type] = true                   // Heisen er i ro, må finne retning og starte bevegelse
		var pair DirnBehaviourPair = requests_chooseDirection(elevator) // Velg retning basert på forespørsler
		elevator.Dirn = pair.dirn                                       // Oppdater retning
		elevator.Behaviour = pair.behaviour                             // Oppdater tilstand
		switch pair.behaviour {
		case EB_DoorOpen: // Hvis heisen skal stoppe i etasjen den står i
			elevio.SetDoorOpenLamp(true)
			TimerStart(elevator.DoorOpenDuration_s)
			elevator = requests_clearAtCurrentFloor(elevator)

		case EB_Moving: // Hvis heisen skal starte bevegelse
			elevio.SetMotorDirection(GetMotorDirectionFromDirn(elevator.Dirn)) // Start motoren

		case EB_Idle:
		}
	}

	setAllLights(elevator) // Oppdater lysindikatorene

	fmt.Println("\nNew state:")
}

func Fsm_onFloorArrival(newFloor int) {
	fmt.Printf("\n\nfsm_onFloorArrival(%d)\n", newFloor)
	elevator.Floor = newFloor

	elevio.SetFloorIndicator(elevator.Floor)

	switch elevator.Behaviour {
	case EB_Moving:
		if requests_shouldStop(elevator) { // Hvis heisen skal stoppe i etasjen den er i, enten fordi request i riktig retning i etasjen eller cab, eller ingen flere forespørsler
			elevio.SetMotorDirection(elevio.MD_Stop)          // Stopp motoren
			elevio.SetDoorOpenLamp(true)                      // Åpne døren
			elevator = requests_clearAtCurrentFloor(elevator) // Rydd opp forespørslene i etasjen
			TimerStart(elevator.DoorOpenDuration_s)
			setAllLights(elevator)
			elevator.Behaviour = EB_DoorOpen
		}
	default:
	}

	fmt.Println("\nNew state:")
}

func Fsm_onDoorTimeout() {
	fmt.Printf("\n\nfsm_onDoorTimeout()\n")

	switch elevator.Behaviour {
	case EB_DoorOpen:
		// Velg retning basert på forespørsler
		var pair DirnBehaviourPair = requests_chooseDirection(elevator)
		elevator.Dirn = pair.dirn
		elevator.Behaviour = pair.behaviour

		switch elevator.Behaviour {
		case EB_DoorOpen:
			// Restart dørtimeren
			TimerStart(elevator.DoorOpenDuration_s)

			// Rydd opp forespørslene i nåværende etasje
			elevator = requests_clearAtCurrentFloor(elevator)

			// Oppdater alle lysindikatorer
			setAllLights(elevator)

		case EB_Moving:
			// Start motoren
			elevio.SetDoorOpenLamp(false) // Slå av dørindikatoren
			elevio.SetMotorDirection(GetMotorDirectionFromDirn(elevator.Dirn))
			fmt.Printf("Motor started moving in direction: %v\n", elevator.Dirn)

		case EB_Idle:
			// Ingen forespørsler igjen, sett til idle
			elevio.SetDoorOpenLamp(false)
		}

	default:
		// Ingenting å gjøre hvis tilstanden ikke er EB_DoorOpen
	}

	fmt.Println("\nNew state:")
}
