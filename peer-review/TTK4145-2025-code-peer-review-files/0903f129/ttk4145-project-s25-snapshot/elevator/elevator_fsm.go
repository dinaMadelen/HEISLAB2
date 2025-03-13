package elevator

import (
	"Project/config"
	"Project/elevator/elevio"
	"fmt"
	"time"
)

func RunElevatorFSM(elevid string,
					localOrderRequest chan<- Order,
					addToLocalQueue <-chan Order,
					orderServed chan<- OrderUpdate) {
	doorTimer := time.NewTimer(config.DOOR_OPEN_TIME) // New timer created with duration DOOR_OPEN_TIME
	if !doorTimer.Stop() {
		<-doorTimer.C
	}
	port := config.BROADCAST_PORT
	addr := "localhost:" + fmt.Sprint(port)

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)
	
	elevio.Init(addr, config.NUM_FLOORS)
	e := ElevatorInit() 
	setAllLights(e)
	if elevio.GetFloor() == -1 { 					// If the elevator is between floors
		elevio.SetMotorDirection(elevio.MD_Down) 	// Return the elevator to the nearest floor
		e.Dir = Down
		e.State = Moving
	initLoop:
		for {
			select {
			case <-drv_floors:
				elevio.SetMotorDirection(elevio.MD_Stop)
				e.Floor = elevio.GetFloor()
				elevio.SetFloorIndicator(e.Floor)
				e.State = Idle
				e.Dir, e.State = chooseDirection(e)
				elevio.SetMotorDirection(elevio.MotorDirection(e.Dir))
				break initLoop

			case <-time.After(5 * time.Second):
				fmt.Println("Waiting for floor sensor")
			}
		}
	}
	for {
		select {
		case btnPress := <-drv_buttons:
			localOrderRequest <- Order{Floor: btnPress.Floor, Button: ButtonType(btnPress.Button)}

		case order := <-addToLocalQueue:
			e.Queue[order.Floor][order.Button] = true
			switch e.State {
			case Idle:
				e.Dir, e.State = chooseDirection(e) 

				switch e.State {
				case DoorOpen:
					elevio.SetDoorOpenLamp(true)
					doorTimer.Reset(config.DOOR_OPEN_TIME)
					clearRequestsAtFloor(&e)
				case Moving:
					elevio.SetMotorDirection(elevio.MotorDirection(e.Dir))
				case Idle:
				}

			case DoorOpen:
				if shouldClearImmediately(e, order.Floor, elevio.ButtonType(order.Button)) {
					doorTimer.Reset(config.DOOR_OPEN_TIME)
					orderServed <- OrderUpdate{Floor: order.Floor, Button: ButtonType(order.Button), Served: true}
				} else {
					e.Queue[order.Floor][order.Button] = true
				}

			case Moving:
				e.Queue[order.Floor][order.Button] = true
			}
			setAllLights(e)

		case floor := <-drv_floors:
			e.Floor = floor
			elevio.SetFloorIndicator(e.Floor)

			if e.State == Moving && shouldStop(e) {
				elevio.SetMotorDirection(elevio.MD_Stop)
				elevio.SetDoorOpenLamp(true)
				doorTimer.Reset(config.DOOR_OPEN_TIME)
				clearRequestsAtFloor(&e)
				setAllLights(e)
				e.State = DoorOpen
			}

		case <-drv_obstr:
			if e.State == DoorOpen {
				if elevio.GetObstruction() {
					elevio.SetMotorDirection(elevio.MD_Stop)
					elevio.SetDoorOpenLamp(true)
					<-drv_obstr
					if !elevio.GetObstruction() {
						doorTimer.Reset(config.DOOR_OPEN_TIME)
					}
				}
			}
		// Stop Button - no functionality implemented
		case <-drv_stop:
			for f := 0; f < config.NUM_FLOORS; f++ {
				for b := elevio.ButtonType(0); b < config.NUM_BUTTONS; b++ {
					elevio.SetButtonLamp(b, f, false)
				}
			}

		case <-doorTimer.C:
			if e.State == DoorOpen {
				newDir, newState := chooseDirection(e)
				e.Dir, e.State = newDir, newState
				switch e.State {
				case DoorOpen:
					elevio.SetDoorOpenLamp(true)
					clearRequestsAtFloor(&e)
					setAllLights(e)
					doorTimer.Reset(config.DOOR_OPEN_TIME)
				case Moving:
					elevio.SetDoorOpenLamp(false)
					elevio.SetMotorDirection(elevio.MotorDirection(e.Dir))
				case Idle:
					elevio.SetDoorOpenLamp(false)
					elevio.SetMotorDirection(elevio.MD_Stop)
				}
			} 
		}
	}
}

func requests_clearAtCurrentFloor(e_old Elevator, onCleared func(elevio.ButtonType, int)) Elevator {
	e := e_old
	for btn := elevio.ButtonType(0); btn < N_BUTTONS; btn++ {
		if e.Queue[e.Floor][btn] {
			e.Queue[e.Floor][btn] = false
			if onCleared != nil {
				onCleared(btn, e.Floor)
			}
		}
	}
	return e
}

// Returns the time it takes for the elevator to reach the floor of the button press
func TimeToServeRequest(e_copy Elevator, btnPress elevio.ButtonEvent) time.Duration {
	duration := 0 * time.Second

	elevatorArrival := 0

	e := e_copy
	e.Queue[btnPress.Floor][btnPress.Button] = true // Add the button press to the queue
	fmt.Println("Queue: ", e.Queue)

	// Function to be called when a request is cleared
	// Sets elevatorArrival to 1 if the button press is cleared
	onCleared := func(btn elevio.ButtonType, floor int) {
		if btn == btnPress.Button && floor == btnPress.Floor {
			elevatorArrival = 1
		}
	}
	switch e.State {
	case Idle:
		e.Dir, e.State = chooseDirection(e)
		if e.Dir == Stop {
			return duration // Elevator is already at the floor
		}
	case Moving:
		duration += config.TRAVEL_TIME / 2
		e.Floor += int(e.Dir)
	case DoorOpen:
		duration -= config.DOOR_OPEN_TIME / 2
		if !requestsAbove(e) && !requestsBelow(e) {
			return duration
		}
	}
	for {
		if shouldStop(e) {
			e = requests_clearAtCurrentFloor(e, onCleared)
			if elevatorArrival == 1 {
				return duration
			}
			duration += config.DOOR_OPEN_TIME
			e.Dir, _ = chooseDirection(e)
		}
		e.Floor += int(e.Dir)
		duration += config.TRAVEL_TIME
	}
}
