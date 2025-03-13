package driver

import (
	"TTK4145---project/config"
	hra "TTK4145---project/cost_fns"
	"TTK4145---project/driver-go/elevio"
	"time"
)

func removeOrder(floor, button int) {
	UpdateQueue(floor, button, config.NoOrder)
	elevio.SetButtonLamp(elevio.ButtonType(button), floor, false)
}

func decideDir() {

	if config.ElevatorInstance.Floor == 0 && config.ElevatorInstance.Direction == elevio.MD_Down {
		config.ElevatorInstance.Direction = elevio.MD_Stop
		elevio.SetMotorDirection(elevio.MD_Stop)
		return
	}
	if config.ElevatorInstance.Floor == config.NumFloors-1 && config.ElevatorInstance.Direction == elevio.MD_Up {
		config.ElevatorInstance.Direction = elevio.MD_Stop
		elevio.SetMotorDirection(elevio.MD_Stop)
		return
	}

	if config.ElevatorInstance.State == config.DoorOpen {
		elevio.SetMotorDirection(elevio.MD_Stop)
		return
	}

	queue := <-config.MyQueue

	// Check if there is an order at the current floor and stop
	for i := 0; i < config.NumButtons; i++ {
		if queue[config.ElevatorInstance.Floor][i] {
			removeOrder(config.ElevatorInstance.Floor, i)
			elevio.SetMotorDirection(elevio.MD_Stop)
			go openDoor()
			break
		}

	}

	// Check if there are orders above or below the current floor
	for i := 0; i < config.NumFloors; i++ {
		if queue[i][config.ButtonUp] || queue[i][config.ButtonDown] || queue[i][config.ButtonCab] {
			if i > config.ElevatorInstance.Floor {
				config.ElevatorInstance.State = config.Moving
				config.ElevatorInstance.Direction = elevio.MD_Up
				elevio.SetMotorDirection(elevio.MD_Up)
				break
			} else if i < config.ElevatorInstance.Floor {
				config.ElevatorInstance.State = config.Moving
				config.ElevatorInstance.Direction = elevio.MD_Down
				elevio.SetMotorDirection(elevio.MD_Down)
				break
			}

		}
	}

}

func reachedFloor() bool {
	queue := <-config.MyQueue
	for i := 0; i < config.NumButtons; i++ {
		if queue[elevio.GetFloor()][i] {
			return true
		}
	}
	return false
}

func openDoor() {
	elevio.SetMotorDirection(elevio.MD_Stop)
	config.ElevatorInstance.Direction = elevio.MD_Stop
	elevio.SetDoorOpenLamp(true)
	config.ElevatorInstance.State = config.DoorOpen
	time1 := time.Now()
	for {
		if time.Since(time1) > 3*time.Second {
			break
		}
	}
	elevio.SetDoorOpenLamp(false)
	config.ElevatorInstance.State = config.Idle
	decideDir()
}

func UpdateQueue(floor, button int, state config.OrderState) {
	config.ElevatorInstance.Queue[floor][button] = state
	hra.HRA()
}
