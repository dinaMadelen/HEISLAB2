package fsm

import (
	"fmt"
	"main/elev_algo_go/elevator"
	"main/elev_algo_go/requests_elev"
	"main/elev_algo_go/timer"
	"main/elevio"
)

var Elevator_cab elevator.Elevator

func Fsm_init() {
	Elevator_cab = elevator.Elevator_uninitialised()
}

func Fsm_set_all_lights(es elevator.Elevator) {
	for floor := 0; floor < elevator.N_FLOORS; floor++ {
		for btn := 0; btn < elevator.N_BUTTONS; btn++ {
			// outputDevice.requestButtonLight(floor, btn, es.Requests[floor][btn])
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, bool(es.Requests[floor][btn]))
		}
	}
}

func Fsm_on_init_between_floors() {
	// outputDevice.motorDirection(ecom/mikgje/ttk4145/elevio"levio.MD_Down)
	elevio.SetMotorDirection(elevio.MD_Down)
	Elevator_cab.Dirn = elevio.MD_Down
	Elevator_cab.Behaviour = elevator.EB_Moving
}

func Fsm_on_request_button_press(btn_floor int, btn_type elevio.ButtonType, timer_channel chan<- bool) {
	elevator.Elevator_print(Elevator_cab)

	switch Elevator_cab.Behaviour {
	case elevator.EB_DoorOpen:
		if requests_elev.Requests_should_clear_immediately(Elevator_cab, btn_floor, btn_type) {
			go timer.Timer_start2(Elevator_cab.Config.DoorOpenDuration_s, timer_channel)
		} else {
			Elevator_cab.Requests[btn_floor][btn_type] = true
		}
		break

	case elevator.EB_Moving:
		Elevator_cab.Requests[btn_floor][btn_type] = true
		break

	case elevator.EB_Idle:
		Elevator_cab.Requests[btn_floor][btn_type] = true
		pair := requests_elev.Requests_choose_direction(Elevator_cab)
		Elevator_cab.Dirn = pair.Dirn
		Elevator_cab.Behaviour = pair.Behaviour
		switch pair.Behaviour {
		case elevator.EB_DoorOpen:
			// outputDevice.doorLight(1)
			elevio.SetDoorOpenLamp(true)
			go timer.Timer_start2(Elevator_cab.Config.DoorOpenDuration_s, timer_channel)
			Elevator_cab = requests_elev.Requests_clear_at_current_floor(Elevator_cab)
			break

		case elevator.EB_Moving:
			// outputDevice.motorDirection(Elevator_cab.Dirn)
			elevio.SetMotorDirection(Elevator_cab.Dirn)
			break

		case elevator.EB_Idle:
			break
		}
	}

	Fsm_set_all_lights(Elevator_cab)

	fmt.Println("\nNew state:")
	elevator.Elevator_print(Elevator_cab)
}

func Fsm_on_floor_arrival(new_floor int, timer_channel chan<- bool) {
	elevator.Elevator_print(Elevator_cab)
	Elevator_cab.Floor = new_floor
	// outputDevice.floorIndicator(Elevator_cab.Floor)
	elevio.SetFloorIndicator(Elevator_cab.Floor)

	switch Elevator_cab.Behaviour {
	case elevator.EB_Moving:
		if requests_elev.Requests_should_stop(Elevator_cab) {
			// outputDevice.motorDirection(elevio.MD_Stop)
			elevio.SetMotorDirection(elevio.MD_Stop)
			// outputDevice.doorLight(1)
			elevio.SetDoorOpenLamp(true)
			Elevator_cab = requests_elev.Requests_clear_at_current_floor(Elevator_cab)
			go timer.Timer_start2(Elevator_cab.Config.DoorOpenDuration_s, timer_channel)
			Fsm_set_all_lights(Elevator_cab)
			Elevator_cab.Behaviour = elevator.EB_DoorOpen
		}
		break
	default:
		break
	}

	fmt.Println("\nNew state:")
	elevator.Elevator_print(Elevator_cab)
}

func Fsm_on_door_timeout(timer_channel chan<- bool) {
	elevator.Elevator_print(Elevator_cab)

	elevio.SetDoorOpenLamp(false) // for å cleare lampen ved timeout

	switch Elevator_cab.Behaviour {
	case elevator.EB_DoorOpen:
		pair := requests_elev.Requests_choose_direction(Elevator_cab)
		Elevator_cab.Dirn = pair.Dirn
		Elevator_cab.Behaviour = pair.Behaviour

		switch Elevator_cab.Behaviour {
		case elevator.EB_DoorOpen:
			go timer.Timer_start2(Elevator_cab.Config.DoorOpenDuration_s, timer_channel)
			Elevator_cab = requests_elev.Requests_clear_at_current_floor(Elevator_cab)
			Fsm_set_all_lights(Elevator_cab)
			break
		case elevator.EB_Moving:
			elevio.SetMotorDirection(Elevator_cab.Dirn)
			break
		case elevator.EB_Idle:
			// outputDevice.doorLight(0)
			elevio.SetDoorOpenLamp(false)
			// outputDevice.motorDirection(elevator.Dirn)
			elevio.SetMotorDirection(Elevator_cab.Dirn)
			break
		}

		break
	default:
		break
	}

	fmt.Print("\nNew state:")
	elevator.Elevator_print(Elevator_cab)
}
