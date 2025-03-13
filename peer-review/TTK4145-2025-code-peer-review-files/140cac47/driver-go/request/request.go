package request

import (
	"Driver-go/config"
	"Driver-go/elevator"
	"Driver-go/elevio"
)


func RequestAbove(e elevator.Elevator) bool{
	for floor := e.Floor+1; floor < config.NumFloors; floor++{
		for button := 0; button < config.NumButtons; button++{
			if e.Requests[floor][button]{
				return true
			}
		}
	}
	return false
}

func RequestBelow(e elevator.Elevator)bool {
	for floor := 0; floor < e.Floor; floor++{
		for btn := range e.Requests[floor]{
			if e.Requests[floor][btn]{
				return true
			}
		}
	}
	return false
}

//Clears the requests at the current floor going in the same direction
func RequestClearAtCurrentFloor(e *elevator.Elevator){
	e.Requests[e.Floor][int(elevio.BT_Cab)] = false
	switch{
	case e.Direction == elevio.MD_Up:
		e.Requests[e.Floor][int(elevio.BT_HallUp)] = false
		if !RequestAbove(*e){
			e.Requests[e.Floor][int(elevio.BT_HallDown)] = false
		}
	case e.Direction == elevio.MD_Down:
		e.Requests[e.Floor][int(elevio.BT_HallDown)] = false
		if !RequestBelow(*e){
			e.Requests[e.Floor][int(elevio.BT_HallUp)] = false
		}
	}
}

//Stop based on current floor and direction
func RequestShouldStop(e *elevator.Elevator)bool {
	switch{
	case e.Direction == elevio.MD_Down:
		return e.Requests[e.Floor][int(elevio.BT_HallDown)] || e.Requests[e.Floor][int(elevio.BT_Cab)] || !RequestBelow(*e)
	case e.Direction == elevio.MD_Up:
		return e.Requests[e.Floor][int(elevio.BT_HallUp)] || e.Requests[e.Floor][int(elevio.BT_Cab)] || !RequestAbove(*e)
	default:
		return true
	}
}

//Chooses direction based on the the requests made 
func RequestChooseDirection(e *elevator.Elevator){
	switch e.Direction{
	case elevio.MD_Up:
		if RequestAbove(*e){
			e.Direction = elevio.MD_Up
		} else if RequestBelow(*e){
			e.Direction = elevio.MD_Down
		} else{
			e.Direction = elevio.MD_Stop
		}
	case elevio.MD_Down:
		fallthrough
	
	case elevio.MD_Stop:
		if RequestBelow(*e){
			e.Direction = elevio.MD_Down
		} else if RequestAbove(*e){
			e.Direction = elevio.MD_Up
		} else{
			e.Direction = elevio.MD_Stop
		}
	}

}



func RequestClearHall(e *elevator.Elevator){
	for floor := 0; floor < config.NumFloors; floor++{
		for btn := 0; btn < config.NumButtons; btn++{
			e.Requests[floor][btn] = false
		}
	}
}

