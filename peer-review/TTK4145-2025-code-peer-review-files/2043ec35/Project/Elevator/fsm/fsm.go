package fsm

import (
	//"fmt"
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/elevio"
	"elevproj/Elevator/requests"
	"time"
)

func OnRequestButtonPress(e elevator.Elevator, btnFloor int, btnType elevio.ButtonType, timerStart_chan chan time.Duration) elevator.Elevator {
	newElevator := elevator.DeepCopyElevator(e)
	switch newElevator.Behaviour {
	case elevator.EB_dooropen:
		if requests.ShouldClearImmediately(newElevator, btnFloor, btnType) {
			timerStart_chan <- newElevator.DoorOpenDuration
		} else {
			newElevator.Requests[btnFloor][btnType] = true
		}

	case elevator.EB_moving:
		newElevator.Requests[btnFloor][btnType] = true

	case elevator.EB_idle:
		newElevator.Requests[btnFloor][btnType] = true
		println("skal bestemme retning")
		pair := requests.ChooseDirection(newElevator)
		newElevator.Dirn = pair.Dirn
		newElevator.Behaviour = pair.Behaviour

		switch pair.Behaviour {
		case elevator.EB_dooropen:
			elevio.SetDoorOpenLamp(true)
			timerStart_chan <- newElevator.DoorOpenDuration
			newElevator = requests.ClearAtCurrentFloor(newElevator)

		case elevator.EB_moving:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(newElevator.Dirn)

		case elevator.EB_idle:
			elevio.SetDoorOpenLamp(false)
		}

	}
	elevator.SetAllLights(newElevator)
	return newElevator
}

func OnFloorArrival(e elevator.Elevator, newFloor int, timerStart_chan chan time.Duration) elevator.Elevator {
	newElevator := elevator.DeepCopyElevator(e)
	newElevator.LatestFloor = newFloor
	elevio.SetFloorIndicator(newElevator.LatestFloor)
	switch newElevator.Behaviour {
	case elevator.EB_moving:
		if requests.ShouldStop(newElevator) {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			newElevator = requests.ClearAtCurrentFloor(newElevator)
			timerStart_chan <- newElevator.DoorOpenDuration
			newElevator.Behaviour = elevator.EB_dooropen
			elevator.SetAllLights(newElevator)
		}
	default:
		break
	}
	return newElevator
}

func OnDoorTimeout(e elevator.Elevator, timerStart_chan chan time.Duration) elevator.Elevator {
	newElevator := elevator.DeepCopyElevator(e)
	elevio.SetDoorOpenLamp(false)
	pair := requests.ChooseDirection(newElevator)
	newElevator.Dirn = pair.Dirn
	newElevator.Behaviour = pair.Behaviour

	switch newElevator.Behaviour {
	case elevator.EB_dooropen:
		timerStart_chan <- newElevator.DoorOpenDuration
		newElevator = requests.ClearAtCurrentFloor(newElevator)
		elevator.SetAllLights(newElevator)

	case elevator.EB_moving:
		elevio.SetDoorOpenLamp(false)
		elevio.SetMotorDirection(newElevator.Dirn)
		newElevator = requests.ClearAtCurrentFloor(newElevator)
	case elevator.EB_idle:
		elevio.SetDoorOpenLamp(false)
		elevio.SetMotorDirection(newElevator.Dirn)
		newElevator = requests.ClearAtCurrentFloor(newElevator)

	}
	return newElevator

}

func SetNewRequest(newBtn elevio.ButtonEvent, elev elevator.Elevator, timerStart_chan chan time.Duration) elevator.Elevator {
	switch newBtn.Button {
	case elevio.BT_Cab:

		elev.Requests[newBtn.Floor][newBtn.Button] = true
		elev = OnRequestButtonPress(elev, newBtn.Floor, newBtn.Button, timerStart_chan)
	}
	if newBtn.Floor != -1 {
		if newBtn.Button != elevio.BT_Cab {
			elev.Requests[newBtn.Floor][newBtn.Button] = true
			elev = OnRequestButtonPress(elev, newBtn.Floor, newBtn.Button, timerStart_chan)

		} else if (elev.LatestFloor == newBtn.Floor && elev.Behaviour == elevator.EB_moving) || elev.LatestFloor != newBtn.Floor {
			elev.Requests[newBtn.Floor][newBtn.Button] = true
			elev = OnRequestButtonPress(elev, newBtn.Floor, newBtn.Button, timerStart_chan)
		}
	}
	return elev

}
