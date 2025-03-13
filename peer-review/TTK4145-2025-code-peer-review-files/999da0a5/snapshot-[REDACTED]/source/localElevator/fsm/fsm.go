package fsm

// This module should contain the finite state machine for the local elevator

import (
	. "source/config"
	"source/localElevator/elevio"
	"source/localElevator/requests"
	"time"
	"fmt"
)

func ShouldStop(elev Elevator) bool {
	switch elev.Direction {
	case UP:
		if elev.Floor==NUM_FLOORS-1{
			return true
		}else{
			return elev.Orders[elev.Floor][elevio.BT_HallUp] || 
			elev.Orders[elev.Floor][elevio.BT_Cab] || 
			!requests.OrdersAbove(elev)
		}
	case DOWN:
		if elev.Floor==0{
			return true
		}else{
			return elev.Orders[elev.Floor][elevio.BT_HallDown] || 
			elev.Orders[elev.Floor][elevio.BT_Cab] || 
			!requests.OrdersBelow(elev)
		}
	case STOP:
		return true
	}
	return false
}

func ChooseDirection(elev Elevator) int {
	// In case of orders above and below; choose last moving direction
	if elev.PrevDirection == UP{
		if requests.OrdersAbove(elev) {
			return UP
		} else if requests.OrdersBelow(elev) {
			return DOWN
		}
	} else {
		if requests.OrdersBelow(elev) {
			return DOWN
		} else if requests.OrdersAbove(elev) {
			return UP
		}
	}
	return STOP

}

//Simulates elevator execution and returns approx time until pickup at NewOrder.Floor
func TimeUntilPickup(elev Elevator, NewOrder Order) time.Duration{
	duration := time.Duration(0)
	elev.Orders[NewOrder.Floor][NewOrder.Button]=true
	// Determines initial state
	switch elev.State {
	case IDLE:
		elev.Direction = ChooseDirection(elev)
		if elev.Direction == STOP && elev.Floor == NewOrder.Floor{
			return duration
		}
	case MOVING:
		duration += T_TRAVEL / 2
		elev.Floor += int(elev.Direction)
	case DOOR_OPEN:
		duration -= T_DOOR_OPEN / 2
	}

	for {
		if ShouldStop(elev) {
			if elev.Floor == NewOrder.Floor{
				return duration
			}else{
				for btn:=0; btn<NUM_BUTTONS; btn++{
					elev.Orders[elev.Floor][btn]=false
				}
				duration += T_DOOR_OPEN
				elev.Direction = ChooseDirection(elev)
			}
		}
		elev.Floor += int(elev.Direction)
		duration += T_TRAVEL
	}
}

func Run(
	elev *Elevator, 
	ElevChan chan <-Elevator, 
	AtFloorChan <-chan int, 
	OrderChan <-chan Order,
	hallLightsRXChan <-chan [][]bool,
	ObstructionChan <-chan bool,
	myId string) {

	ElevChan <- *elev
	HeartbeatTimer := time.NewTimer(T_HEARTBEAT)
	DoorTimer := time.NewTimer(T_DOOR_OPEN)
	DoorTimer.Stop()
	ObstructionTimer := time.NewTimer(T_OBSTRUCTED_LOCAL)
	ObstructionTimer.Stop()
	
	for {
		select {
		case NewOrder := <-OrderChan:
			if NewOrder.Id == myId{
				elev.Orders[NewOrder.Floor][NewOrder.Button] = true
				switch elev.State {
				case IDLE:
					elev.Direction = ChooseDirection(*elev)
					elevio.SetMotorDirection(elevio.MotorDirection(elev.Direction))
					if elev.Direction == STOP {
						elevio.SetDoorOpenLamp(true)
						DoorTimer.Reset(T_DOOR_OPEN)
						//If order is at same floor, take order after opening door.
						//Be carefull! Maybe this should be done after the door closes!
						//i.e. at case <- DoorTimer.C
						//What if someone obstructs the door so it cannot close after the order is accepted by an elev
						//Intrduce a timer for that order. If not taken within 5 sec, redistribute. (Primary stuff)
						elev.Orders[elev.Floor][NewOrder.Button] = false
						if(NewOrder.Button == int(elevio.BT_Cab)){
							elevio.SetButtonLamp(elevio.BT_Cab, NewOrder.Floor, false)
						}
						elev.State = DOOR_OPEN
					} else {
						elev.State = MOVING
					}
				case MOVING: //NOOP
				case DOOR_OPEN:
					if elev.Floor == NewOrder.Floor {
						elev.Orders[elev.Floor][NewOrder.Button] = false
						elevio.SetButtonLamp(elevio.ButtonType(NewOrder.Button), elev.Floor, false)
						if !elev.Obstructed{
							DoorTimer.Reset(T_DOOR_OPEN)
						}
					}
				}
				ElevChan <- *elev
			}
		
		case hallLights := <- hallLightsRXChan:
			for floor := range hallLights { // Iterate over floors
				for btn := range hallLights[floor] { // Iterate over buttons
					elevio.SetButtonLamp(elevio.ButtonType(btn), floor, hallLights[floor][btn])
				}
			}

		case elev.Floor = <-AtFloorChan:
			elevio.SetFloorIndicator(elev.Floor)
			if ShouldStop(*elev) {
				elevio.SetMotorDirection(elevio.MD_Stop)
				requests.ClearOrder(elev, elev.Floor)
				elev.Direction = STOP
				elevio.SetDoorOpenLamp(true)
				DoorTimer.Reset(T_DOOR_OPEN)
				elev.State = DOOR_OPEN
			}
			ElevChan <- *elev

		case <-DoorTimer.C:

			elevio.SetDoorOpenLamp(false)
			elev.Direction = ChooseDirection(*elev)
			if elev.Direction == STOP {
				elev.State = IDLE
			} else {
				elevio.SetMotorDirection(elevio.MotorDirection(elev.Direction))
				elev.State = MOVING
			}
			ElevChan <- *elev
		
		case ObsEvent:= <-ObstructionChan:
			fmt.Println("Obstruction switch")
			if elev.State==DOOR_OPEN{
				switch ObsEvent{
					case true:
						elev.Obstructed = true
						DoorTimer.Stop()
						ObstructionTimer.Reset(T_OBSTRUCTED_LOCAL)
					case false:
						elev.Obstructed = false
						DoorTimer.Reset(T_DOOR_OPEN)
				}
			}
			ElevChan <- *elev
		
		case <- ObstructionTimer.C:
			//Delete active hall orders
			for floor, floorOrders := range(elev.Orders){
				for btn, orderActive := range(floorOrders){
					if orderActive && btn != int(elevio.BT_Cab) {
						elev.Orders[floor][btn] = false
					}
				}
			}
			ObstructionTimer.Stop()

		case <-HeartbeatTimer.C:
			ElevChan <- *elev
			HeartbeatTimer.Reset(T_HEARTBEAT)
		}

		time.Sleep(T_SLEEP)
	}
}