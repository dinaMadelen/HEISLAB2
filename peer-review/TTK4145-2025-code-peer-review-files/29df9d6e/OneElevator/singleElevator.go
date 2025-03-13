package main

// FSM for singleElevator

import (
	"OneElevator/elevio"
	"fmt"
	"time"
)

type ElevOutputDevice struct{}

var (
	elevator     Elevator
	outputDevice ElevOutputDevice
)

func SingleElevator(ButtonPressCh chan elevio.ButtonEvent, FloorSensorCh chan int, StopButtonCh chan bool, ObstructionSwitchCh chan bool) {
	// Initialize the elevator instance
	elevator = ElevatorUninitialized()

	// Check if elevator starts between floors
	if initialFloor := elevio.GetFloor(); initialFloor == -1 {
		fmt.Println("Elevator is between floors on startup. Running initialization...")
		fsm_onInitBetweenFloors()
	} else {
		// If the elevator starts at a valid floor, initialize its state
		fsm_onFloorArrival(initialFloor)
	}

	// Polling rate configuration
	inputPollRate := 25 * time.Millisecond // Adjust as needed

	// Initialize system state
	obstructionActive := false
	lastKnownDirection := elevio.MotorDirection(0)
	stop := false

	// Main event loop
	for {
		select {
		case buttonEvent := <-ButtonPressCh:
			// Handle button press event
			if stop {
				elevio.SetMotorDirection(lastKnownDirection)
				stop = false
			}

			fmt.Printf("Button pressed at floor %d, button type %s\n", buttonEvent.Floor, buttonEvent.Button)
			fsm_onRequestButtonPress(buttonEvent.Floor, buttonEvent.Button)

		case currentFloor := <-FloorSensorCh:
			// Handle floor sensor event
			fmt.Printf("[SingleElevator] : Floor %d\n", currentFloor)

			if elevator.Floor != currentFloor {
				fmt.Printf("Arrived at floor %d\n", currentFloor)
				fsm_onFloorArrival(currentFloor)
				elevio.SetFloorIndicator(currentFloor)

				if !obstructionActive {
					timerStop()
					timerStart(3.0)
					fmt.Println("timer started")
				}
			}
			obstructionActive = false

		case stopPressed := <-StopButtonCh:
			// Handle stop button event
			if stopPressed {
				lastKnownDirection = elevator.Dirn
				fmt.Println("Stop button pressed!")
				fmt.Println(lastKnownDirection)
				elevio.SetStopLamp(true)
				elevio.SetMotorDirection(0)
				stop = true
			} else {
				fmt.Println("Stop button released!")
				elevio.SetStopLamp(false)
			}

		case <-time.After(inputPollRate):
			// Periodic tasks (check timer)
			if timerTimedOut() {
				fmt.Println("\nDoor timeout occurred.\n")
				fsm_onDoorTimeout()
				timerStop()
			}

		case obstruction := <-ObstructionSwitchCh:
			if obstruction {
				obstructionActive = true
				timerStop()
				fmt.Println("obstruction switch")
			} else if !obstruction {
				obstructionActive = false
				timerStop()
				timerStart(3.0)
				fmt.Println("obstruction switch off")
			}
		}
	}
	select {}
}

func setAllLights(e Elevator) {
	for floor := 0; floor < elevio.NumFloors; floor++ {
		for btn := 0; btn < elevio.NumButtonTypes; btn++ {
			state := e.Requests[floor][btn]
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, state == 1)
		}
	}
}

func fsm_onInitBetweenFloors() {
	// Sett motoren til å bevege seg nedover for å komme i definert tilstand
	elevio.SetMotorDirection(elevio.MD_Down)

	// Oppdater heisens retning og oppførsel
	elevator.Dirn = elevio.MD_Down
	elevator.Behaviour = EB_Moving
}

func fsm_onRequestButtonPress(btnFloor int, btnType elevio.ButtonType) {
	fmt.Printf("fsm_onRequestButtonPress(%d, %s)", btnFloor, btnType)

	switch elevator.Behaviour {
	case EB_DoorOpen:
		if ShouldClearImmediately(elevator, btnFloor, btnType) {
			timerStart(elevator.Config.DoorOpenDuration)
		} else {
			elevator.Requests[btnFloor][btnType] = 1
		}

	case EB_Moving:
		elevator.Requests[btnFloor][btnType] = 1

	case EB_Idle:
		elevator.Requests[btnFloor][btnType] = 1
		dirnBehaviour := ChooseDirection(elevator)
		elevator.Dirn = dirnBehaviour.Dirn
		elevator.Behaviour = dirnBehaviour.Behaviour

		switch dirnBehaviour.Behaviour {
		case EB_DoorOpen:
			elevio.SetDoorOpenLamp(true)
			timerStart(elevator.Config.DoorOpenDuration)
			elevator = ClearAtCurrentFloor(elevator)

		case EB_Moving:
			elevio.SetMotorDirection(elevator.Dirn)

		case EB_Idle:
			// Ingen handling nødvendig
		}
	}

	// Oppdater knappelysene
	setAllLights(elevator)

	// Logg den nye tilstanden til heisen
	fmt.Println("\nNew state:", elevator.Behaviour)
}

func fsm_onFloorArrival(newFloor int) {
	fmt.Printf("\nfsm_onFloorArrival(%d)\n", newFloor)

	// Oppdater heisens nåværende etasje
	elevator.Floor = newFloor

	// Oppdater etasjeindikatoren
	elevio.SetFloorIndicator(elevator.Floor)

	switch elevator.Behaviour {
	case EB_Moving:
		// Sjekk om heisen skal stoppe i denne etasjen
		if ShouldStop(elevator) {
			// Stopp motoren
			elevio.SetMotorDirection(elevio.MD_Stop)
			fmt.Println("Motor stopped")

			// Slå på dørlyset
			elevio.SetDoorOpenLamp(true)
			fmt.Println("Door open lamp on")

			// Rydd forespørsler for nåværende etasje
			elevator = ClearAtCurrentFloor(elevator)
			fmt.Println("Requests cleared at current floor")

			// Start timer for å holde dørene åpne
			timerStart(elevator.Config.DoorOpenDuration)
			fmt.Println("Timer started for door open duration")

			// Oppdater knappelysene
			setAllLights(elevator)
			fmt.Println("Button lights updated")

			// Endre heisens oppførsel til "DoorOpen"
			elevator.Behaviour = EB_DoorOpen
			fmt.Println("Elevator behaviour set to DoorOpen")
		}

	default:
		// Ingen spesifikk handling for andre oppførsler
	}

	// Logg den nye tilstanden til heisen
	fmt.Println("\nNew state:", elevator.Behaviour)
}

func fsm_onDoorTimeout() {
	fmt.Printf("fsm_onDoorTimeout()")

	switch elevator.Behaviour {
	case EB_DoorOpen:
		// Velg neste retning og oppførsel basert på forespørsler
		dirnBehaviour := ChooseDirection(elevator)
		elevator.Dirn = dirnBehaviour.Dirn
		elevator.Behaviour = dirnBehaviour.Behaviour

		switch elevator.Behaviour {
		case EB_DoorOpen:
			// Start timer på nytt og rydd forespørsler i nåværende etasje
			timerStart(elevator.Config.DoorOpenDuration)
			elevator = ClearAtCurrentFloor(elevator)
			setAllLights(elevator)

		case EB_Moving, EB_Idle:
			// Lukk dørene og sett motorretning
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevator.Dirn)
		}

	default:
		// Ingen handling for andre tilstander
	}

	fmt.Println("\nNew state:", elevator.Behaviour)
}
