package inits

import (
	. "source/config"
	"source/localElevator/elevio"
)

func ElevatorInit(elev *Elevator, id string){
	currentFloor := elevio.GetFloor()
	if currentFloor == -1{
		ch:=make(chan int)
		go elevio.PollFloorSensor(ch)
		elevio.SetMotorDirection(elevio.MD_Down)
		
		select{case currentFloor = <-ch:}
		elevio.SetMotorDirection(elevio.MD_Stop)
	}
	elev.Id = id
  elev.PrevDirection = DOWN
	elev.Direction = int(elevio.MD_Stop)
	elev.State = IDLE
	elev.Floor = currentFloor
	elev.Obstructed = false
	elevio.SetFloorIndicator(elev.Floor)
}

func LightsInit(){
	for fl:=0; fl<NUM_FLOORS; fl++{
		for btn:=0; btn<NUM_BUTTONS; btn++{
			elevio.SetButtonLamp(elevio.ButtonType(btn),fl,false)
		}
	}
}