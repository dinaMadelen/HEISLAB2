package fsm

import (
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
	"github.com/Eirik-a-Johansen/trippel_elevator/handleOrders"
	"github.com/Eirik-a-Johansen/trippel_elevator/timer"
)

/*
This module contains the state machine for each elevator
This state machine determines the action of a single elevator

*/



func Fsm(e *elevator.Elevator) {
	for {
		switch e.Behaviour {
		case elevator.EB_Idle:
			moveToValidFloor(e) //avoid error

			targetFloor := handleOrders.FindTargetFloor(*e)

			//checking for trigger actions
			if e.Stop {
				e.Behaviour = elevator.EB_Stop
				continue
			} else if targetFloor != -1 {
				e.Behaviour = elevator.EB_Moving
				continue
			}
		case elevator.EB_Moving:
			targetFloor := handleOrders.FindTargetFloor(*e)

			moveTo(targetFloor, e)

			e.Behaviour = elevator.EB_DoorOpen
			continue
		case elevator.EB_DoorOpen:
			moveToValidFloor(e)

			openDoor(e)

			if !closeDoor(e) {
				continue
			}

			handleOrders.ClearOrders(e, e.Floor)
			e.Behaviour = elevator.EB_Idle
		}
		time.Sleep(300 * time.Millisecond)
	}
}

func closeDoor(e *elevator.Elevator) bool {
	counter := 0
	for e.DoorObstruction {
		counter++
		if counter == 10 {
			e.Functional = false
		}
		time.Sleep(100 * time.Millisecond)
	}
	e.Functional = true
	e.OpenDoor = false
	driver.SetDoorOpenLamp(false)
	return true
}

func openDoor(e *elevator.Elevator) {
	timer.Timer_start(3)
	e.OpenDoor = true
	driver.SetDoorOpenLamp(true)

	for {
		if e.DoorObstruction {
			timer.Timer_stop()
			counter := 0
			for e.DoorObstruction {
				counter++
				if counter == 10 {
					e.Functional = false
				}
				time.Sleep(100 * time.Millisecond)
			}
			timer.Timer_start(3)
			e.Functional = true
		}
		if timer.Timer_TimedOut() {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
}

// Makes sure the elevator is on a floor.
// Keeps the direction of the elevator if it has one
func moveToValidFloor(e *elevator.Elevator) {
	timer.Timer_start(5) //detects error if movement takes over 5 seconds

	motorDirection := driver.MD_Stop
	if driver.GetFloor() == -1 {
		if e.Dirn == elevator.D_Up {
			motorDirection = driver.MD_Up
		} else {
			motorDirection = driver.MD_Down
		}

		for driver.GetFloor() == -1 {
			driver.SetMotorDirection(motorDirection)
			time.Sleep(100 * time.Millisecond)

			if timer.Timer_TimedOut() {
				e.Functional = false
			}
		}
		e.Functional = true

	}
	driver.SetMotorDirection(driver.MD_Stop)
	e.Floor = driver.GetFloor()
	driver.SetFloorIndicator(e.Floor)
}

// Moves the elevator to a given floor
// Checks while moving if it should stop at an earlier floor
func moveTo(targetFloor int, e *elevator.Elevator) {

	if targetFloor < 0 || targetFloor >= driver.N_Floors {
		return
	}

	//Update the direction of the elevator
	motorDirection := driver.MD_Stop

	direction := e.Floor - targetFloor
	if direction > 0 {
		e.Dirn = elevator.D_Down
		motorDirection = driver.MD_Down
	} else if direction < 0 {
		e.Dirn = elevator.D_Up
		motorDirection = driver.MD_Up
	} else {
		if e.Dirn == elevator.D_Up {
			e.Dirn = elevator.D_Down
		} else if e.Dirn == elevator.D_Down {
			e.Dirn = elevator.D_Up
		}
	}

	//return if the elevator does not need to move
	if targetFloor == e.Floor {
		return
	}

	timer.Timer_start(10) // detect error if movement takes over 10 seconds

	currentFloor := e.Floor
	newTargetFloor := targetFloor
	for currentFloor != targetFloor {
		driver.SetMotorDirection(motorDirection)

		//check if new order has arrived on the way
		newTargetFloor = handleOrders.FindTargetFloor(*e)

		if newTargetFloor != -1 && newTargetFloor != targetFloor {
			targetFloor = newTargetFloor
		}

		currentFloor = driver.GetFloor()

		if currentFloor != -1 {
			e.Floor = currentFloor
			driver.SetFloorIndicator(currentFloor)
		}

		if timer.Timer_TimedOut() {
			e.Functional = false
		}

		time.Sleep(100 * time.Millisecond)
	}
	driver.SetMotorDirection(driver.MD_Stop)
	e.Functional = true
}
