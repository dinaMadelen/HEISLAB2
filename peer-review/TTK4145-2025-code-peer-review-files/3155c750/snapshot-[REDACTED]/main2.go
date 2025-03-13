package main

import (
	"fmt"
	elevalgo "root/elevator"
	"root/elevio"
	"time"
	"root/reciver"
	"root/transmitter"
)

var elevator_1_ip = "localhost"


func main() {
	fmt.Println("Started!")
	go transmitter.Start_tcp_call("8080", elevator_1_ip)
	reciver.Start_tcp_listen("8081")
	/*
	go utility.Start_tcp_call2("8081", elevator_2_ip) // for the third elevator
	utility.Start_tcp_listen2("8081")
	*/
	time.Sleep(5*time.Second)
	elevio.Init("localhost:12346", elevalgo.NUM_FLOORS)

	elevalgo.MakeFsm()

	
	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	poll_timer := make(chan bool)
	update_recived := make(chan [3]int)
	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevalgo.PollTimer(poll_timer)
	go reciver.Listen_recive(update_recived)
	transmitter.Send_Elevator_data(elevalgo.GetElevatordata())

	for {
		select {
		case button := <-drv_buttons:
			elevalgo.FsmOnRequestButtonPress(button.Floor, elevalgo.Button(button.Button))
			e := elevalgo.GetElevatordata()
			go transmitter.Send_Elevator_data(e) //skal flystes 
			elevalgo.SetAllLights()
			
		case floor := <-drv_floors:
			if !elevalgo.IsDoorObstructed() {
				elevalgo.FsmOnFloorArrival(floor)
			}
		case obstructed := <-drv_obstr:
			if obstructed {
				elevalgo.DoorObstructed()
			} else {
				elevalgo.DoorUnobstructed()
			}
		case <-poll_timer:
			if !elevalgo.IsDoorObstructed() {
				elevalgo.StopTimer()
				elevalgo.FsmOnDoorTimeout()
			} else {
				elevalgo.StartTimer()
			}
		case update := <-update_recived:
			elevalgo.UpdatesharedHallRequests(update)
			elevalgo.SetAllLights()

		}
	}
}

