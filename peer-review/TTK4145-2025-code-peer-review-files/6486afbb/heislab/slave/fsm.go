package slave

import (
	"fmt"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
)

/*
	The main finite state machine of the elevator.
	Explained in more detail in the README

Input: the elevator ID and all relevant channels
*/
func fsm_fsm(ID int, tx chan<- EventMessage, ordersRx <-chan [config.N_FLOORS][config.N_BUTTONS]bool,
	drv_floors <-chan int, drv_obstr <-chan bool, drv_stop <-chan bool, t_start chan int, t_end *time.Timer) {
	//initialize elevator
	var elevator Elevator
	elevator.ID = ID
	io_updateLights(elevator.Requests)

	n_elevator := fsm_onInit(elevator)
	elevator = elevator_updateElevator(n_elevator, elevator, tx, t_start)

	for {
		fmt.Println("FSM:New Loop")
		select {
		case newRequests := <-ordersRx:
			fmt.Println("Slave: Updating orders")

			elevator.Requests = newRequests
			n_elevator = fsm_onRequests(elevator)
			elevator = elevator_updateElevator(n_elevator, elevator, tx, t_start)

		case floor := <-drv_floors:
			fmt.Println("FSM: Floor arrival", floor)
			n_elevator = fsm_onFloorArrival(floor, elevator)
			elevator = elevator_updateElevator(n_elevator, elevator, tx, t_start)

		case obs := <-drv_obstr:
			n_elevator = fsm_onObstruction(obs, elevator)
			elevator = elevator_updateElevator(n_elevator, elevator, tx, t_start)

		case <-drv_stop:
			fsm_onStopButtonPress()

		case <-t_end.C:
			fmt.Println("FSM: Timer end")
			n_elevator = fsm_onTimerEnd(elevator)
			elevator = elevator_updateElevator(n_elevator, elevator, tx, t_start)
		}
	}
}

/*
	Activates when the elevator is initialized

Input: the old elevator object

Returns: the new elevator object with updated direction and behaviour
*/
func fsm_onInit(elevator Elevator) Elevator {
	fmt.Println("onInit")
	elevator.Direction = D_Down
	elevator.Behaviour = EB_Moving
	fmt.Println("offInit")
	return elevator
}

/*
	Activates when the elevator receives new requests

Input: the old elevator object with updated requests

Returns: the new elevator object with updated direction and behaviour
*/
func fsm_onRequests(elevator Elevator) Elevator {
	fmt.Println("onRequest")
	switch elevator.Behaviour {
	case EB_Idle:
		direction, behaviour := requests_chooseDirection(elevator)
		elevator.Direction = direction
		elevator.Behaviour = behaviour
		if elevator.Behaviour == EB_DoorOpen {
			elevator = requests_clearAtCurrentFloor(elevator)
		}
	}
	return elevator
}

/*
	Activates when the elevator floor sensor is triggered

Input: the new floor and the old elevator object

Returns: the new elevator object
*/
func fsm_onFloorArrival(newFloor int, elevator Elevator) Elevator {
	elevator.Stuck = false //if the elevator arrives at a floor, it is not stuck
	fmt.Println("onFloorArrival")
	elevator.Floor = newFloor
	switch elevator.Behaviour {
	case EB_Moving:
		if requests_shouldStop(elevator) { //This causes the door to open on init, probably fine?
			elevator = requests_clearAtCurrentFloor(elevator)
			elevator.Behaviour = EB_DoorOpen
		}
	}
	return elevator
}

/*
	Activates when the obstruction sensor is triggered

Input: the old elevator object with updated obstruction status and behaviour

Returns: the new elevator object
*/
func fsm_onObstruction(obstruction bool, elevator Elevator) Elevator {
	fmt.Println("onObstruction")
	elevator.Stuck = obstruction
	if obstruction {
		elevator.Behaviour = EB_DoorOpen
		elevator.Direction = D_Stop
	} else {
		direction, behaviour := requests_chooseDirection(elevator)
		elevator.Direction = direction
		elevator.Behaviour = behaviour
	}
	return elevator
}

/*
	Activates when the stop button sensor is triggered

Does nothing but print a message
*/
func fsm_onStopButtonPress() {
	fmt.Println("You pressed the stop button :)")
}

/*
	Activates when the timer ends
	Either the door should close or the elevator is stuck

Input: the old elevator object

Returns: the new elevator object
*/
func fsm_onTimerEnd(elevator Elevator) Elevator {

	switch elevator.Behaviour {
	case EB_DoorOpen:
		fmt.Println("FSM:onTimerEnd DO")

		if !elevator.Stuck {
			direction, behaviour := requests_chooseDirection(elevator)
			elevator.Direction = direction
			elevator.Behaviour = behaviour
			if elevator.Behaviour == EB_DoorOpen {
				elevator = requests_clearAtCurrentFloor(elevator)
			}
		}
	case EB_Moving:
		fmt.Println("FSM:onTimerEnd M")
		elevator.Stuck = true
	}
	return elevator
}
