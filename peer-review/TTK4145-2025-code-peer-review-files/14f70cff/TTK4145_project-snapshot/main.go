package main

import (
	"elevator/backup"
	"elevator/distribution"
	. "elevator/elevio"
	"elevator/fsm"
	"elevator/timer"
	"flag"
	"fmt"
	"time"
)

func PollDoorTimer(e *Elevator, obstruction chan bool) {
	for {
		if <-obstruction {
			timer.TimerStart()
		} else {
			if timer.TimerTimedOut() {
				fmt.Println("Stopping timer")
				timer.TimerStop()
				fsm.FsmOnDoorTimeout(e)

			}
		}
	}
}

func main() {
	var simPort string
	flag.StringVar(&simPort, "port", "", "port of the server")
	flag.Parse()
	numFloors := 4

	var elevator = backup.Backup( /*sim_port*/ )

	Init(fmt.Sprintf("localhost:%s", simPort), numFloors)

	elevator_state := make(chan Elevator)
	drv_buttons := make(chan ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	obstruction_chan := make(chan bool)

	go backup.TransformToPrimary(elevator_state, simPort)
	go func() {
		for {
			elevator_state <- elevator
			time.Sleep(1 * time.Second)
		}
	}()

	go PollButtons(drv_buttons)
	go PollFloorSensor(drv_floors)
	go PollObstructionSwitch(drv_obstr)
	go PollStopButton(drv_stop)
	go PollDoorTimer(&elevator, obstruction_chan)
	go distribution.RunNetwork()

	if fsm.IsElevatorUninitialized(elevator) {
		fsm.FsmOnInitBetweenFloors(&elevator, drv_floors)
	} else if elevator.Behaviour == EB_DoorOpen {
		fsm.FsmOnDoorTimeout(&elevator)
	} else {
		SetMotorDirection(elevator.Dirn)
	}

	prev := -1

	for {
		select {
		case b := <-drv_buttons:
			fsm.FsmOnRequestButtonPress(b.Floor, b.Button, &elevator)
		case floor_sensed := <-drv_floors:
			if floor_sensed != -1 && floor_sensed != prev {
				fsm.FsmOnFloorArrival(floor_sensed, &elevator)
			}

			prev = floor_sensed

		case obstr := <-drv_obstr:
			obstruction_chan <- obstr
		}

	}
}
