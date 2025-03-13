package fsm

import (
	. "Driver-go/elevator"
	. "Driver-go/elevio"
	. "Driver-go/requests"
	. "Driver-go/timer"
	"fmt"
)

// Lager en global heis
var FSMElevator Elevator

// Initialiserer heisen
func FsmInit() {
	FSMElevator = ElevatorUninitialized()
}

// Skrur på rette lys
func SetAllLights(e Elevator) {
	for floor := 0; floor < NFloors; floor++ {
		for btn := 0; btn < NBtns; btn++ {
			SetButtonLamp(ButtonType(btn), floor, e.Requests[floor][btn] == 1)
		}
	}
}

// Initialisering hvis heis er mellom etasjer
func Fsm_onInitBetweenFloors() {
	fmt.Println("between floors")
	SetMotorDirection(MotorDirection(Down))
	FSMElevator.Dirn = Down
	FSMElevator.Behaviour = Moving
}

// Funksjon for når en knapp trykkes
func Fsm_onRequestButtonPress(btnFloor int, btnType ButtonType) {
	//Skriver ut hvilken knapp som ble trykket
	fmt.Printf("\n\nfsm_onRequestButtonPress(%d, %d)\n", btnFloor, btnType)
	ElevatorPrint(FSMElevator)

	switch FSMElevator.Behaviour {
	case DoorOpen:
		//Dersom døren er åpen, og en bestilling til den nåværende etasjen kommer inn, slettes den umiddelbart og dørtimeren restarter
		if RequestsShouldClearImmediately(FSMElevator, btnFloor) {
			TimerStart(ElevatorUninitialized().Config.DoorOpenDuration_s)
		} else {
			//Hvis døren er åpen og bestillingen er en annen etasje, legges den til vanlig
			FSMElevator.Requests[btnFloor][btnType] = 1
		}
	//Dersom heisen er i bevegelse så legges alle bestillinger til
	case Moving:
		FSMElevator.Requests[btnFloor][btnType] = 1
	//Dersom heisen er i hvilemodus, så legges bestillingen til og bevegelsesretning bestemmes
	case Idle:
		FSMElevator.Requests[btnFloor][btnType] = 1
		pair := RequestsChooseDirection(FSMElevator)
		FSMElevator.Dirn = pair.Direction
		FSMElevator.Behaviour = pair.Behaviour

		switch pair.Behaviour {
		case DoorOpen:
			SetDoorOpenLamp(true)
			TimerStart(FSMElevator.Config.DoorOpenDuration_s)
			FSMElevator = RequestsClearAtCurrentFloor(FSMElevator)

		case Moving:
			switch FSMElevator.Dirn {
			case Up:
				SetMotorDirection(MotorDirection(Up))
			case Down:
				SetMotorDirection(MotorDirection(Down))
			case Stop:
				SetMotorDirection(MotorDirection(Stop))
			}

		case Idle:

		}
	}

}

// Funksjon for hvs som skal skje når heisen ankommer en etasje
func FsmOnFloorArrival(newFloor int) {
	// Logg funksjonskallet
	fmt.Printf("\n\nfsmOnFloorArrival(%d)\n", newFloor)
	// Skriv ut heisens nåværende tilstand
	ElevatorPrint(FSMElevator)

	// Oppdater heisens nåværende etasje
	FSMElevator.Floor = newFloor

	//Oppdater gulvindikatoren i heisen
	SetFloorIndicator(FSMElevator.Floor)

	// Håndter tilstanden basert på heisens adferd
	switch FSMElevator.Behaviour {
	case Moving:
		// Sjekk om heisen skal stoppe i denne etasjen
		if RequestsShouldStop(FSMElevator) {
			fmt.Println("Should stop at current floor")
			SetMotorDirection(MotorDirection(Stop))
			fmt.Print(Stop)
			// Slå på dørenes lys
			SetDoorOpenLamp(true)
			// Fjern forespørsler for denne etasjen
			FSMElevator = RequestsClearAtCurrentFloor(FSMElevator)
			// Start dørtimeren
			TimerStart(FSMElevator.Config.DoorOpenDuration_s)
			// Oppdater knappelysene
			SetAllLights(FSMElevator)
			// Sett tilstanden til "Dør åpen"
			FSMElevator.Behaviour = DoorOpen
		}
	default:
		// Gjør ingenting for andre tilstander
	}

	// Logg den nye tilstanden
	fmt.Println("\nNew state:")
	ElevatorPrint(FSMElevator)
}

func FsmOnDoorTimeout() {
	// Logg funksjonskallet
	fmt.Println("\n\nfsmOnDoorTimeout()")
	// Skriv ut heisens nåværende tilstand
	ElevatorPrint(FSMElevator)
	SetDoorOpenLamp(false)
	// Håndter tilstanden basert på heisens adferd
	switch FSMElevator.Behaviour {
	case DoorOpen:
		// Velg ny retning og oppførsel basert på forespørsler
		pair := RequestsChooseDirection(FSMElevator)
		FSMElevator.Dirn = pair.Direction
		FSMElevator.Behaviour = pair.Behaviour
		fmt.Print("Døra er nå åpen i fsmdoortimedout")
		// Håndter den nye oppførselen
		switch FSMElevator.Behaviour {
		case DoorOpen:
			// Fortsett med åpen dør hvis nødvendig
			TimerStart(FSMElevator.Config.DoorOpenDuration_s)
			FSMElevator = RequestsClearAtCurrentFloor(FSMElevator)
			SetAllLights(FSMElevator)
			for 1 != 0 {
				if TimerTimedOut() {
					SetDoorOpenLamp(false)
					break
				}
			}

		case Moving, Idle:
			// Lukk døren og sett motorretningen
			SetDoorOpenLamp(false)
			switch FSMElevator.Dirn {
			case Up:
				SetMotorDirection(MotorDirection(Up))
			case Down:
				SetMotorDirection(MotorDirection(Down))
			case Stop:
				SetMotorDirection(MotorDirection(Stop))
			}
		}

	default:
		// Gjør ingenting for andre tilstander
		fmt.Print("Default i fsmdoortimedout?")
	}

	// Logg den nye tilstanden
	fmt.Println("\nNew state:")
	ElevatorPrint(FSMElevator)
}
