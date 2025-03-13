package fsmpkg

import (
	"Driver-go/elevio"
	"elevator"
	"elevator_io_device"
	"fmt"
	"request"
	"time"
)

// Elevator FSM struct
type FSM struct {
	El elevator.Elevator
	Od elevator_io_device.ElevOutputDevice
}

// Set all elevator lights
func (fsm *FSM) SetAllLights() {
	print("Setting all lights\n")
	for floor := 0; floor < elevator.N_FLOORS; floor++ {
		for btn := elevio.ButtonType(0); btn < elevator.N_BUTTONS; btn++ {
			fsm.Od.RequestButtonLight(floor, btn, fsm.El.RequestsToDo[floor][btn])
		}
	}
}

// Handle initialization between floors
func (fsm *FSM) Fsm_onInitBetweenFloors() {
	fsm.Od.MotorDirection(elevio.MD_Up)
	fsm.El.Dirn = elevio.MD_Up
	fsm.El.Behaviour = elevator.EB_Moving
}

// Handle button press event
func (fsm *FSM) Fsm_onRequestButtonPress(btn_floor int, btn_type elevio.ButtonType, start_timer chan time.Duration) {
	fmt.Printf("\n\n(%d, %s)\n", btn_floor, btn_type)
	elevator.Elevator_print(fsm.El)

	switch fsm.El.Behaviour {
	case elevator.EB_DoorOpen:
		if request.Requests_shouldClearImmediately(fsm.El, btn_floor, btn_type) {
			start_timer <- fsm.El.Config.DoorOpenDuration
		} else {
			fsm.El.RequestsToDo[btn_floor][btn_type] = true
		}

	case elevator.EB_Moving:
		fsm.El.RequestsToDo[btn_floor][btn_type] = true

	case elevator.EB_Idle:
		fsm.El.RequestsToDo[btn_floor][btn_type] = true
		pair := request.Requests_chooseDirection(fsm.El)
		fsm.El.Dirn = pair.Dirn
		fsm.El.Behaviour = pair.Behaviour

		switch pair.Behaviour {
		case elevator.EB_DoorOpen:
			elevio.SetDoorOpenLamp(false)
			start_timer <- fsm.El.Config.DoorOpenDuration
			fsm.El = request.Requests_clearAtCurrentFloor(fsm.El)

		case elevator.EB_Moving:
			fsm.Od.MotorDirection(fsm.El.Dirn)

		case elevator.EB_Idle:
			// Do nothing
		}
	}

	fsm.SetAllLights()
	fmt.Println("\nNew state:")
	elevator.Elevator_print(fsm.El)
}

// Handle floor arrival event
func (fsm *FSM) Fsm_onFloorArrival(newFloor int, start_timer chan time.Duration) {
	fmt.Printf("\n\n(%d)\n", newFloor)
	elevator.Elevator_print(fsm.El)

	// fsm.El.PrevFloor = fsm.El.Floor
	fsm.El.Floor = newFloor

	elevio.SetFloorIndicator(newFloor)

	

	switch fsm.El.Behaviour {
	case elevator.EB_Moving:
		if request.Requests_shouldStop(fsm.El) {
			fmt.Printf("Elevator stopping at floor %d \n", fsm.El.Floor)
			fsm.Od.MotorDirection(elevio.MD_Stop)
			//fsm.El = request.Requests_clearAtCurrentFloor(fsm.El)
			elevio.SetDoorOpenLamp(true)
			//fsm.SetAllLights()
			start_timer <- fsm.El.Config.DoorOpenDuration
			fmt.Print("Started doorOpen timer")
			fsm.El.Behaviour = elevator.EB_DoorOpen
			// Send beskjed til master 
		}
	}

	fmt.Println("\nNew state:")
	elevator.Elevator_print(fsm.El)
}

// Handle door timeout event
func (fsm *FSM) Fsm_onDoorTimeout(start_timer chan time.Duration) {
	elevator.Elevator_print(fsm.El)
	
	// fsm.SetAllLights()

	switch fsm.El.Behaviour {
	case elevator.EB_DoorOpen:
		pair := request.Requests_chooseDirection(fsm.El)
		fsm.El.Dirn = pair.Dirn
		fsm.El.Behaviour = pair.Behaviour

		switch fsm.El.Behaviour {
		case elevator.EB_DoorOpen:
			start_timer <- fsm.El.Config.DoorOpenDuration
			fsm.El = request.Requests_clearAtCurrentFloor(fsm.El)
			fsm.SetAllLights()
		case elevator.EB_Moving, elevator.EB_Idle:
			elevio.SetDoorOpenLamp(false)
			fsm.Od.MotorDirection(fsm.El.Dirn)
		}

	}

	fmt.Println("\nNew state:")
	elevator.Elevator_print(fsm.El)
}
