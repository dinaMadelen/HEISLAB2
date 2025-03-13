package elevator

import (
	"fmt"
	"time"
	"project/sanntid/elevator/elevio"
	req "project/sanntid/requests/request"
)

// STATE
//--------------------------------------------------
type ElevatorFunctions interface {
	ChooseDirection()
	OpenDoor() *time.Timer
	FindFloor(drv_floors chan<- int)
	StateMachine(id string, Tx chan req.Request, Rx chan req.Request)
}

type Obs struct {
	Status		bool
	TimeAlive	time.Time
}

type State struct {
	Id			string
	Floor		int
	Direction	elevio.MotorDirection
	Behaviour	elevio.ElevatorBehaviour
	Request		req.Request
	Obstruction	Obs
	Stop		bool
	Busy		bool
}

func InitState(id string, n int) State {
	elevator := State {
		Id:				id,
		Floor:			-1,
		Direction:		elevio.MD_Stop,
		Behaviour:		elevio.EB_Idle,
		Obstruction:	Obs{Status: false, TimeAlive: time.Now(),},
		Stop:			false,
		Busy:			false,
	}

	return elevator
}

func (elev *State) ChooseDirection() {
	if elev.Floor < elev.Request.Button.Floor {
		// Elevator moves up
		elevio.SetMotorDirection(elevio.MD_Up)
		elev.Direction = elevio.MD_Up
	} else if elev.Floor > elev.Request.Button.Floor {
		// Elevator moves down
		elevio.SetMotorDirection(elevio.MD_Down)
		elev.Direction = elevio.MD_Down
	}
}

func (elev *State) OpenDoor() *time.Timer {
	elevio.SetMotorDirection(elevio.MD_Stop)
	elev.Direction = elevio.MD_Stop

	elevio.SetDoorOpenLamp(true)
	elev.Behaviour = elevio.EB_DoorOpen

	// Create timer
	return time.NewTimer(3 * time.Second)
}

func (elev *State) FindFloor(drv_floors chan int) {
	elevio.SetMotorDirection(elevio.MD_Up)
	for {
		select {
			case floor := <-drv_floors:
				elev.Floor = floor
				elevio.SetMotorDirection(elevio.MD_Stop)
				return
		}
	}
}

func (elev *State) StateMachine(reqAssigner chan req.Request, reqUpdate chan req.Request) {
	var doorOpenTimer *time.Timer

	// Create channels
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	// Start subroutines
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	// Find initial floor
	elev.FindFloor(drv_floors)

	for {
		// Avoid dereferncing doorOpenTimer when its nil
		var timeChan <-chan time.Time
		if doorOpenTimer != nil {
			timeChan = doorOpenTimer.C
		}

		select {
			case request := <-reqAssigner:
				fmt.Println("Executing Requeset")
				fmt.Println(request.Button)
				elev.Request = request
				elev.Busy = true

				// Choose initial Direction
				fmt.Println(fmt.Sprintf("our: %d, req: %d", elev.Floor, request.Button.Floor))
				if elev.Floor == elev.Request.Button.Floor {
					doorOpenTimer = elev.OpenDoor()
				} else {
					elev.ChooseDirection()
				}

			// If floor sensor detects a floor
			case floor := <-drv_floors:
				elev.Floor = floor
				elevio.SetFloorIndicator(floor)

				if elev.Floor == elev.Request.Button.Floor {
					doorOpenTimer = elev.OpenDoor()
				} else {
					elev.ChooseDirection()
				}



			case <-timeChan:
				// Check for obstruction
				if elev.Obstruction.Status {
					if elev.Obstruction.TimeAlive.After(time.Now().Add(9 * time.Second)) {
						fmt.Println("What: obstruction not disapearing")
					} else {
						doorOpenTimer = time.NewTimer(3 * time.Second)
					}
				} else {
					// Close door
					elevio.SetDoorOpenLamp(false)
					elev.Behaviour = elevio.EB_Idle

					// Update task to finished
					elev.Request.Status = 3

					// Send updated task status to request handler
					reqUpdate <- elev.Request
				}

			case obstr := <-drv_obstr:
				fmt.Println("Obstruction")
				elev.Obstruction.Status = obstr
				elev.Obstruction.TimeAlive = time.Now()

			case stop_btn := <-drv_stop:
				fmt.Println("Stop")
				elev.Stop = stop_btn
		}
	}
}


