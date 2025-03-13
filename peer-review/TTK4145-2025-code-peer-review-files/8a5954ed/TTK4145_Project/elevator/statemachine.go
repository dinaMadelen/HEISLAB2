package elevator

import (
	"config"
	"elevio"
	"time"
)

func (e *Elevator) State_machine() {
	for {
		switch e.Current_state {
		case State.Undefined:
			e.Starting_routine()

		case State.Idle:
			if e.Queue.Length() > 0 {
				e.Current_state = State.Moving
			}

		case State.Moving:
			e.Choose_target_floor()
			e.Move_to_floor()

		case State.FloorStop:
			e.Remove_order(e.Current_floor)
			// Open door
			e.Door_timer.Reset(config.Door_open_duration)
			e.Current_state = State.DoorOpen

		case State.FullStop:
			elevio.SetMotorDirection(elevio.MD_Stop)
			e.Direction = elevio.MD_Stop
			time.Sleep(2 * time.Second)
			e.Current_state = State.Undefined

		case State.DoorOpen:
			elevio.SetDoorOpenLamp(true)
			// Reset timer if obstruction
			if e.Obstruction {
				e.Door_timer.Reset(config.Door_open_duration)
			}

			if !e.Obstruction && e.Door_timer_done {
				e.Door_timer_done = false
				elevio.SetDoorOpenLamp(false)
				if e.Queue.Length() > 0 {
					e.Current_state = State.Moving
				} else {
					e.Current_state = State.Idle
				}
			}
		}
	}
}

func (e *Elevator) Starting_routine() {
	for elevio.GetFloor() == -1 {
		elevio.SetMotorDirection(elevio.MD_Down)
	}
	e.Current_floor = elevio.GetFloor()
	elevio.SetFloorIndicator(e.Current_floor)

	elevio.SetMotorDirection(elevio.MD_Stop)
	e.Direction = elevio.MD_Stop

	e.Current_state = State.Idle
}

func (e *Elevator) Move_to_floor() {
	if elevio.GetFloor() == e.Target_floor {
		e.Current_floor = elevio.GetFloor()

		elevio.SetMotorDirection(elevio.MD_Stop)
		e.Direction = elevio.MD_Stop
		
		e.Current_state = State.FloorStop
	} else if e.Target_floor > e.Current_floor {
		elevio.SetMotorDirection(elevio.MD_Up)
		e.Direction = elevio.MD_Up
	} else if e.Target_floor < e.Current_floor {
		elevio.SetMotorDirection(elevio.MD_Down)
		e.Direction = elevio.MD_Down
	}
}
