package fsm

import (
	"Driver-go/elevio"
	"fmt"
	"sanntids/cmd/localElevator/config"
	"sanntids/cmd/localElevator/elevator"
	"sanntids/cmd/localElevator/requests"
	"sanntids/cmd/localElevator/timer"
)

func setAllLights(e elevator.Elevator) {
	for floor := 0; floor < config.N_FLOORS; floor++ {
		for btn := 0; btn < config.N_BUTTONS; btn++ {
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, e.Requests[floor][btn])
		}
	}
}

func moveToFirstFloor(floor <-chan int) {
	for {
		elevio.SetMotorDirection(elevio.MD_Down)

		currentFloor := <-floor
		fmt.Printf("moveToFirstFloor: got floor = %d\n", currentFloor)
		if currentFloor == 0 {
			elevio.SetMotorDirection(elevio.MD_Stop)
			break
		}
	}
}

func onRequestButtonPress(el *elevator.Elevator, btnFloor int, btnType elevio.ButtonType) {

	switch el.Behaviour {
	case elevator.EB_DoorOpen:
		if requests.Requests_shouldClearImmediately(*el, btnFloor, btnType) {
			timer.TimerStart(el.Config.DoorOpenDuration_s)
		} else {
			el.Requests[btnFloor][btnType] = true
		}

	case elevator.EB_Moving:
		el.Requests[btnFloor][btnType] = true

	case elevator.EB_Idle:
		el.Requests[btnFloor][btnType] = true
		pair := requests.Requests_chooseDirection(*el)
		el.MotorDirection = pair.MotorDirection
		el.Behaviour = pair.Behaviour
		switch pair.Behaviour {
		case elevator.EB_DoorOpen:
			elevio.SetDoorOpenLamp(true)
			timer.TimerStart(el.Config.DoorOpenDuration_s)
			*el = requests.Requests_clearAtCurrentFloor(*el)

		case elevator.EB_Moving:
			elevio.SetMotorDirection(el.MotorDirection)

		case elevator.EB_Idle:
			// Do nothing
		}
	}

	setAllLights(*el)
}

func onFloorArrival(el *elevator.Elevator, newFloor int) {

	el.Floor = newFloor

	elevio.SetFloorIndicator(el.Floor)

	switch el.Behaviour {
	case elevator.EB_Moving:
		if requests.RequestsShouldStop(*el) {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			*el = requests.Requests_clearAtCurrentFloor(*el)
			timer.TimerStart(el.Config.DoorOpenDuration_s)
			setAllLights(*el)
			el.Behaviour = elevator.EB_DoorOpen
		}
	default:
		// Do nothing for other states
	}

}

func onDoorTimeout(el *elevator.Elevator) {

	switch el.Behaviour {
	case elevator.EB_DoorOpen:
		pair := requests.Requests_chooseDirection(*el)
		el.MotorDirection = pair.MotorDirection
		el.Behaviour = pair.Behaviour

		switch el.Behaviour {
		case elevator.EB_DoorOpen:
			timer.TimerStart(el.Config.DoorOpenDuration_s)
			*el = requests.Requests_clearAtCurrentFloor(*el)
			setAllLights(*el)

		case elevator.EB_Moving, elevator.EB_Idle:
			elevio.SetDoorOpenLamp(false)
			elevio.SetMotorDirection(el.MotorDirection)
		}

	default:
		// Do nothing for other states
	}
}

func onObstruction(el *elevator.Elevator, obstruction bool) {
	fmt.Printf("Obstuction: %v,behavior: %v \n", obstruction, el.Behaviour)
	switch {
	case el.Behaviour == elevator.EB_DoorOpen && obstruction:
		fmt.Println("Stopping timer")
		timer.TimerStop()
	case el.Behaviour == elevator.EB_DoorOpen && !obstruction:
		fmt.Println("Starting timer")
		timer.TimerStart(el.Config.DoorOpenDuration_s)
	}
}


func Fsm(
    drv_buttons chan elevio.ButtonEvent,
    drv_floors chan int,
    drv_obstr chan bool,
    drv_stop chan bool,
	stateRequestTx <-chan chan ElevatorState,
) {

    e := elevator.ElevatorInit()

    setAllLights(e)
    elevio.SetFloorIndicator(0)
    elevio.SetDoorOpenLamp(false)
    moveToFirstFloor(drv_floors)


    for {
        select {
        case btn := <-drv_buttons:
            fmt.Println("Button pressed")
            onRequestButtonPress(&e, btn.Floor, btn.Button)

        case floor := <-drv_floors:
            fmt.Printf("Arrived at floor: %v \n", floor)
            onFloorArrival(&e, floor)

        case <-timer.TimeoutChan():
            fmt.Println("Timeout")
            onDoorTimeout(&e)

        case obstruction := <-drv_obstr:
            onObstruction(&e, obstruction)

        case <-drv_stop:
            //Optional - if stop button causes a state change

		case requestChan := <-stateRequestTx: 
			requestChan <- ElevatorState{
				Floor:          e.Floor,
				MotorDirection: e.MotorDirection,
				Behaviour:      e.Behaviour,
			}
        }
    }
}

// ElevatorState struct (no change)
type ElevatorState struct {
    Floor          int
    MotorDirection elevio.MotorDirection
    Behaviour      elevator.ElevatorBehaviour
}