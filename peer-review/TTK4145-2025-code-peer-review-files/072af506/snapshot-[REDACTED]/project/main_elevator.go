package main

import (
	"main/elev_algo_go/fsm"
	"main/elev_algo_go/timer"
	"main/elevio"
	"fmt"
)

//All required variables and channels are declared here

var(
	numFloors = 4
	is_elevator_obstructed = false //Internal flag to keep track of the elevator's obstruction status
	healthy bool = true //Internal flag to keep track of the elevator's health

	floor_channel = make(chan int)
	button_channel = make(chan elevio.ButtonEvent)
	obstruction_channel = make(chan bool)
	timer_channel = make(chan bool)
	obstruction_timer_channel = make(chan bool)
	abort_timer_channel = make(chan bool)
)

// Flags for the elevator's health status
const(
	UNHEALTHY_FLAG int = -1
	HEALTHY_FLAG int = 100
) 


/*
This file contains all the logic of running the elevator itself, and sending/recieving messages to/from the controller.
All requirements to this file can be considered functional (baseline), and is not relevant to the architecture.
*/


func main_elevator() { 


	// INTIALIZATION

	elevio.Init("localhost:15657", numFloors)
	fsm.Fsm_init()

	if elevio.GetFloor() == -1 {
		fsm.Fsm_on_init_between_floors()
	}

	go elevio.PollButtons(button_channel)
	go elevio.PollFloorSensor(floor_channel)
	go elevio.PollObstructionSwitch(obstruction_channel)

	// INTIALIZATION END

	for {
		select {
		case button := <-button_channel: //On button press send the event to the controller to be handled
			elev_to_ctrl_chan <- button

		case floor := <-floor_channel:  //On arrival at the floor update lights, orders and choose direction
			fsm.Fsm_on_floor_arrival(floor, timer_channel)
			elev_to_ctrl_chan <- fsm.Elevator_cab //Send updated elevator status to controller

		case obstruction := <-obstruction_channel: //This case handles the obstruction switch
			is_elevator_obstructed = obstruction //Set the internal flag to the state of the obstruction switch
			if obstruction {
				fmt.Println("Obstruction detected, starting timer")
				go timer.Obstruction_timer(obstruction_timer_duration, obstruction_timer_channel, abort_timer_channel) //Start watchdog for obstruction switch
			} else {

				if !healthy { //If obstruction is removed and the elevator is unhealthy, set healthy and send message to controller with updated status
					healthy = true
					elevio.SetStopLamp(false)
					elev_to_ctrl_chan <- elevio.ButtonEvent{Floor: HEALTHY_FLAG, Button: elevio.BT_Cab}

				} else { //If obstruction is removed and the elevator is not unhealthy, stop timer and continue as normal.
					fmt.Println("Obstruction removed, stopping timer")
					abort_timer_channel <- false
				}
			}

		case <-obstruction_timer_channel: //The obstruction timer has fired, the elevator is inoperable and communicates this to the controller
			healthy = false
			elevio.SetStopLamp(true)
			elev_to_ctrl_button_chan <- elevio.ButtonEvent{Floor: UNHEALTHY_FLAG, Button: elevio.BT_Cab}

		case <-timer_channel: //The door timer has fired, the elevator will check if there are any orders to handle
							  //If the elevator is obstructed it will start a new timer, otherwise it will close the door and continue
			if !(is_elevator_obstructed) {
				fsm.Fsm_on_door_timeout(timer_channel)
			} else {
				go timer.Timer_start2(3, timer_channel)
			}

		case msg := <-ctrl_to_elev_chan: //When controller has new orders it sends them via this channel and the elevator will handle them
			fsm.Fsm_on_request_button_press(msg.Floor, msg.Button, timer_channel)

		}
	}
}
