package fsm

import (
	"Driver-go/config"
	"Driver-go/elevator"
	"Driver-go/elevio"
	"Driver-go/request"
	"time"
)

// Statemachine for running local elevator
func Fsm(
	ch_orderChan chan elevio.ButtonEvent,
	ch_elevatorState chan<- elevator.Elevator,
	ch_clearLocalHallOrders chan bool,
	ch_arrivedAtFloors chan int,
	ch_obstruction chan bool,
	ch_timerDoor chan bool) {

	elev := elevator.InitElevator()
	e := &elev

	elevio.SetDoorOpenLamp(false)


	ch_elevatorState <- *e

	doorTimer := time.NewTimer(time.Duration(config.DoorOpenDuration) * time.Second)
	timerUpdateState := time.NewTicker(time.Duration(config.StateUpdatePeriodsMs) * time.Millisecond)

	// Statemachine defining the elevators state 
	for {
		elevator.SetLocalLights(*e)
		select {
		case order := <-ch_orderChan: // Handles new order
			switch {
			case e.Behave == elevator.DoorOpen:
				if e.Floor == order.Floor {
					doorTimer.Reset(time.Duration(config.DoorOpenDuration) * time.Second)
				} else {
					e.Requests[order.Floor][order.Button] = true
				}

			case e.Behave == elevator.Moving:
				e.Requests[order.Floor][order.Button] = true

			case e.Behave == elevator.Idle:
				if e.Floor == order.Floor {
					elevio.SetDoorOpenLamp(true)
					doorTimer.Reset(time.Duration(config.DoorOpenDuration) * time.Second)
					e.Behave = elevator.DoorOpen
					ch_elevatorState <- *e
				} else {
					e.Requests[order.Floor][order.Button] = true
					request.RequestChooseDirection(e)
					elevio.SetMotorDirection(e.Direction)
					e.Behave = elevator.Moving
					ch_elevatorState <- *e
					break
				}
			}

		case floor := <-ch_arrivedAtFloors: // Handles arriving at floor
			e.Floor = floor
			switch {
			case e.Behave == elevator.Moving:
				if request.RequestShouldStop(e) {
					elevio.SetMotorDirection(elevio.MD_Stop)
					request.RequestClearAtCurrentFloor(e)
					elevio.SetDoorOpenLamp(true)
					doorTimer.Reset(time.Duration(config.DoorOpenDuration) * time.Second)
					e.Behave = elevator.DoorOpen
					ch_elevatorState <- *e
				}
			default:
				break

			}

		case <-doorTimer.C: // Handles door
			switch {
			case e.Behave == elevator.DoorOpen:
				if e.Obstructed {
					elevio.SetMotorDirection(elevio.MD_Stop)
					doorTimer.Stop()
				} else {
					request.RequestChooseDirection(e)
					elevio.SetMotorDirection(e.Direction)
					elevio.SetDoorOpenLamp(false)
					if e.Direction == elevio.MD_Stop {
						e.Behave = elevator.Idle
						ch_elevatorState <- *e
					} else {
						e.Behave = elevator.Moving
						ch_elevatorState <- *e
					}
				}

			default:
				break
			}

		case <-ch_clearLocalHallOrders: // Delete the hallorders of this elevator
			request.RequestClearHall(e)

		case obstruction := <-ch_obstruction: // Handles obstruction
			if obstruction {
				e.Obstructed = true
				elevio.SetDoorOpenLamp(true)
				doorTimer.Stop()
			} else {
				e.Obstructed = false
				doorTimer.Reset(time.Duration(config.DoorOpenDuration) * time.Second)
			}
			ch_elevatorState <- *e

		case <-timerUpdateState.C: // Handles timeout
			ch_elevatorState <- *e
			timerUpdateState.Reset(time.Duration(config.StateUpdatePeriodsMs) * time.Millisecond)

		}
	}
}
