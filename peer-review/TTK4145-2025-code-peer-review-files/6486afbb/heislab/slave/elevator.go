package slave

import (
	"fmt"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/driver-go/elevio"
)

type ElevatorBehaviour int

const (
	EB_Idle ElevatorBehaviour = iota
	EB_DoorOpen
	EB_Moving
)

type ElevatorDirection int

const (
	D_Down ElevatorDirection = -1
	D_Stop ElevatorDirection = 0
	D_Up   ElevatorDirection = 1
)

type Elevator struct {
	ID        int
	Floor     int
	Direction ElevatorDirection
	Requests  [config.N_FLOORS][config.N_BUTTONS]bool
	Behaviour ElevatorBehaviour
	Stuck     bool
}

/*
	Checks if an elevator has states within the correct bounds

Input: The elevator to be checked

Returns: True if the elevator is valid, false otherwise
*/
func elevator_validElevator(elevator Elevator) bool {
	return elevator.Behaviour >= EB_Idle && elevator.Behaviour <= EB_Moving && //Behaviour in bounds
		elevator.Direction >= D_Down && elevator.Direction <= D_Up && //Direction in bounds
		elevator.Floor > -1 && elevator.Floor < config.N_FLOORS && //Floor in bounds
		!elevator.Requests[config.N_FLOORS-1][elevio.BT_HallUp] && !elevator.Requests[0][elevio.BT_HallDown] && //no impossible Requests
		!(elevator.Behaviour == EB_Moving && elevator.Direction == D_Stop) //no Behaviour moving without Direction
}

/*
	Updates the elevator struct with the new elevator struct
	Notifies the master if the elevators stuck status has changed
	Activates the IO of the elevator

Input: The new elevator struct, the old elevator struct, the channel to send messages to the master, the channel to start the timer

Returns: The updated elevator struct
*/
func elevator_updateElevator(n_elevator Elevator, elevator Elevator, tx chan<- EventMessage, t_start chan int) Elevator {
	if elevator_validElevator(n_elevator) {
		fmt.Println("Valid elevator")
		io_activateIO(n_elevator, t_start)

		if n_elevator.Stuck != elevator.Stuck { //if stuck status has changed
			tx <- EventMessage{0, n_elevator, Stuck, elevio.ButtonEvent{}} //send message to master
		}

		if n_elevator.Behaviour == EB_DoorOpen || n_elevator.Floor != elevator.Floor { //if the elevator is at a floor or the door is open
			tx <- EventMessage{0, n_elevator, FloorArrival, elevio.ButtonEvent{}} //send message to master
		}
		return n_elevator
	}
	return elevator
}
