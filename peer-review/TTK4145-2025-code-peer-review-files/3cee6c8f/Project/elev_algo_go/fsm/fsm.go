package fsm

import (
	"Driver-go/elevio"
	"fmt"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/elevator"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/requests"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/timer"
)

var elev elevator.Elevator

func Init() {
	elev.Initialize()
}

func setAllLights(elev elevator.Elevator) {
	for floor := 0; floor < elevator.NumFloors; floor++ {
		for btn := 0; btn < elevator.NumButtons; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, elev.Requests[floor][btn])
		}
	}
}

func OnInitBetweenFloors() {
	fmt.Println("FSM: On init between floors")
	elevio.SetMotorDirection(elevio.MD_Down)
	elev.Dirn = elevio.MD_Down
	elev.Behaviour = elevator.EB_Moving
}

func OnRequestButtonPress(btn_floor int, btn_type elevio.ButtonType) {
	fmt.Println("FSM: On request button press")
	switch elev.Behaviour {
	case elevator.EB_DoorOpen:
		if requests.ShouldClearImmediately(elev, btn_floor, btn_type) {
			timer.Start(float64(elev.Config.DoorOpenDuration_s))
		} else {
			elev.Requests[btn_floor][btn_type] = true
		}
	case elevator.EB_Moving:
		elev.Requests[btn_floor][btn_type] = true
	case elevator.EB_Idle:
		elev.Requests[btn_floor][btn_type] = true

		// Additional logic to handle state transition from Idle
		pair := requests.ChooseDirection(elev)
		elev.Dirn = pair.Dirn
		elev.Behaviour = pair.Behaviour
		switch pair.Behaviour {
		case elevator.EB_DoorOpen:
			elevio.SetDoorOpenLamp(true)
			timer.Start(elev.Config.DoorOpenDuration_s)
			elev = requests.ClearAtCurrentFloor(elev)
		case elevator.EB_Moving:
			elevio.SetMotorDirection(elevio.MotorDirection(elev.Dirn))
		case elevator.EB_Idle:
			// Do nothing
		}
	}
	// Update lights
	setAllLights(elev)
	fmt.Println("\nNew state:\n")
	elev.Elevator_print()
}

func OnFloorArrival(newFloor int) {
	fmt.Println("FSM: On floor arrival")
	fmt.Println("Floor arrived at: ", newFloor)
	elev.Elevator_print()

	elev.Floor = newFloor
	elevio.SetFloorIndicator(elev.Floor)

	switch elev.Behaviour {
	case elevator.EB_Moving:
		if requests.ShouldStop(elev) {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			elev = requests.ClearAtCurrentFloor(elev)
			timer.Start(elev.Config.DoorOpenDuration_s)
			setAllLights(elev)
			elev.Behaviour = elevator.EB_DoorOpen
		}
	default:
		// Do nothing
	}
	fmt.Println("New state: ")
	elev.Elevator_print()
}

func OnDoorTimeout() {
	fmt.Println("FSM: Door timeout")
	elev.Elevator_print()

	switch elev.Behaviour {
	case elevator.EB_DoorOpen:
		pair := requests.ChooseDirection(elev)
		elev.Dirn = pair.Dirn
		elev.Behaviour = pair.Behaviour
		fmt.Println("Timeout before second case")
		fmt.Println("New direction: ", elevator.DirnToString(elev.Dirn))
		fmt.Println("New behaviour: ", elevator.EBToString(elev.Behaviour))
		
		switch elev.Behaviour {
		case elevator.EB_DoorOpen:
			timer.Start(elev.Config.DoorOpenDuration_s)
			elev = requests.ClearAtCurrentFloor(elev)
			setAllLights(elev)
			break
		case elevator.EB_Moving:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevio.MotorDirection(elev.Dirn))
		case elevator.EB_Idle:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(elevio.MotorDirection(elev.Dirn))
			break
		}
	}
	fmt.Println("New state: ")
	elev.Elevator_print()
}

// Something YMSE, IDK
func OnObstruction(obstr bool) {
	fmt.Println("FSM: On obstruction")
	elev.Elevator_print()

	switch elev.Behaviour {
	case elevator.EB_DoorOpen:
		if obstr {
			timer.Stop()
		} else {
			timer.Start(elev.Config.DoorOpenDuration_s)
		}
	}
	fmt.Println("New state: ")
	elev.Elevator_print()
}
