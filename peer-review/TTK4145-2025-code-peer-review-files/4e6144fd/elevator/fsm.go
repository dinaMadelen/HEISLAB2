package elevator

import (
	elevio "Driver-Elevio"
	"fmt"
)

// FSM (Finite State Machine) styrer heisens tilstand og oppførsel basert på knappetrykk, etasjeanløp og dørlukkingshendelser.
// Den håndterer tilstander som Idle, DoorOpen og Moving, og bestemmer heisens retning og handlinger.

// Må opprette en assigner, fjerne funksjoner som bestemmer logikk for valg av etasje - erstattes av kostfunksjonen, beholde logikk som gir retning.
// En kanal for assigning og en for meldinger. Enveis kommunikasjon.

var ElevatorID string
var outputDevice elevio.ElevOutputDevice

func InitFSM() {
	InitSharedState()
	ElevatorID = getElevatorID()
	outputDevice = elevio.GetOutputDevice()

	// Initialize elevator in the global state
	localElevator = ElevatorUninitialized()
	UpdateSharedState()

	fmt.Println("FSM initialized for elevator:", ElevatorID)
}

func convertDirnToMotor(d Dirn) elevio.MotorDirection {
	switch d {
	case "up":
		return elevio.MD_Up
	case "down":
		return elevio.MD_Down
	default:
		return elevio.MD_Stop
	}
}

// **setAllLights**: Oppdaterer alle heisknapper med riktig lysstatus
func setAllLights() {
	sharedState := GetSharedState()
	hallRequests := sharedState.HallRequests
	elevator := GetLocalElevator()

	for floor := 0; floor < N_FLOORS; floor++ {
		// Cab requests
		outputDevice.RequestButtonLight(floor, elevio.BT_Cab, elevator.CabRequests[floor])

		// Hall requests fra SharedState
		outputDevice.RequestButtonLight(floor, elevio.BT_HallUp, hallRequests[floor][B_HallUp])
		outputDevice.RequestButtonLight(floor, elevio.BT_HallDown, hallRequests[floor][B_HallDown])
	}
}

// **FSMStartMoving**: Kalles for å sjekke om heisen skal starte bevegelse
func FSMStartMoving() {
	elevator := GetLocalElevator()
	sharedState := GetSharedState()

	// Hvis heisen er idle og har forespørsler, velg retning og start motor
	if elevator.Behaviour == EB_Idle && hasPendingRequests(elevator, sharedState.HallRequests) {
		elevator.Dirn = requestsChooseDirection(elevator)

		if elevator.Dirn != D_Stop {
			elevator.Behaviour = EB_Moving
			outputDevice.MotorDirection(convertDirnToMotor(elevator.Dirn))
		}

		UpdateLocalElevator(elevator)
		UpdateSharedState()
	}
}

// **FSMOnInitBetweenFloors**: Kalles hvis heisen starter mellom etasjer
func FSMOnInitBetweenFloors() {
	elevator := GetLocalElevator()

	outputDevice.MotorDirection(elevio.MD_Down)
	elevator.Dirn = D_Down
	elevator.Behaviour = EB_Moving

	UpdateLocalElevator(elevator) // Oppdater lokalt
	UpdateSharedState()           // Synkroniser med SharedState
}

// **FSMOnRequestButtonPress**: Kalles når en knapp trykkes
func FSMOnRequestButtonPress(btnFloor int, btnType elevio.ButtonType) {
	fmt.Printf("FSMOnRequestButtonPress(%d, %d)\n", btnFloor, btnType)

	elevator := GetLocalElevator()
	sharedState := GetSharedState()

	// 1. Oppdater lokal tilstand eller SharedState basert på type request
	if btnType == elevio.BT_Cab {
		elevator.CabRequests[btnFloor] = true
		UpdateLocalElevator(elevator) // Kun lokal oppdatering
		UpdateSharedState()           // Synkroniser med SharedState
	} else {
		sharedState.HallRequests[btnFloor][btnType] = true
		GlobalState.mu.Lock()
		GlobalState.HRA = sharedState
		GlobalState.mu.Unlock()
	}

	// 2. Be om ny oppdragsfordeling
	AssignRequestChannel <- struct{}{}
	assignments := <-AssignResultChannel

	// 3. Oppdater SharedState med nye oppdrag
	if assignedRequests, exists := assignments[ElevatorID]; exists {
		GlobalState.mu.Lock()
		GlobalState.HRA.HallRequests = assignedRequests
		GlobalState.mu.Unlock()
		fmt.Printf("Updated assignments for %s: %+v\n", ElevatorID, assignedRequests)
	}

	// 4. Start bevegelse hvis nødvendig
	FSMStartMoving()

	setAllLights()
}

// **FSMOnFloorArrival**: Kalles når heisen ankommer en ny etasje
func FSMOnFloorArrival(newFloor int) {
	fmt.Printf("\nFSMOnFloorArrival(%d)\n", newFloor)

	elevator := GetLocalElevator()
	sharedState := GetSharedState()

	// 1. Oppdater etasjeindikator og lagre ny etasje i lokal state
	elevator.Floor = newFloor
	outputDevice.FloorIndicator(newFloor)
	UpdateLocalElevator(elevator)

	// 2. Sjekk om heisen skal stoppe
	if elevator.Behaviour == EB_Moving {
		if requestsShouldStop(elevator, sharedState.HallRequests) {
			outputDevice.MotorDirection(elevio.MD_Stop)
			outputDevice.DoorLight(true)
			elevator.Behaviour = EB_DoorOpen
			UpdateLocalElevator(elevator)

			// Start dør-timer
			TimerStart(DoorOpenDurationS)

			// 3. Klarer requests på nåværende etasje
			elevator = requestsClearAtCurrentFloor(elevator)
			UpdateLocalElevator(elevator)
			UpdateSharedState() // Synkroniser
			setAllLights()
		}
	}
	UpdateSharedState()
}

// **FSMOnDoorTimeout**: Kalles når dør-timeren utløper
func FSMOnDoorTimeout() {
	fmt.Println("\nFSMOnDoorTimeout()")

	elevator := GetLocalElevator()
	sharedState := GetSharedState()

	// Hvis døren er åpen, bestem neste handling
	if elevator.Behaviour == EB_DoorOpen {
		elevator.Behaviour = EB_Idle

		// Be om ny oppdragsfordeling
		AssignRequestChannel <- struct{}{} // Sender tom struct for å trigge assigner
		assignments := <-AssignResultChannel

		// Oppdater hall requests basert på nye oppdrag
		if assignedRequests, exists := assignments[ElevatorID]; exists {
			GlobalState.mu.Lock()
			GlobalState.HRA.HallRequests = assignedRequests
			GlobalState.mu.Unlock()
		}

		// Bestem om heisen skal fortsette eller gå i idle
		if hasPendingRequests(elevator, sharedState.HallRequests) {
			elevator.Behaviour = EB_Moving
			outputDevice.DoorLight(false)
			outputDevice.MotorDirection(convertDirnToMotor(elevator.Dirn))
		} else {
			elevator.Behaviour = EB_Idle
		}

		UpdateLocalElevator(elevator)
		UpdateSharedState() // Synkroniser
	}
}
