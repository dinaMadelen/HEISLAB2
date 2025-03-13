package elevator

import (
	"context"
	"fmt"
	"time"

	"realtime_systems/elevio"
	fsm "realtime_systems/state_machine"
)

type Direction int

const (
	Up Direction = iota
	Down
	Stop
)

const doorOpenTimeSeconds = 3 * time.Second

type Elevator struct {
	fsm       *fsm.FSM
	floor     int
	doorTimer *time.Timer
	orders    []bool
	direction Direction
}

// Events
type FloorArrival struct{ Floor int }
type ButtonPress struct{ Event elevio.ButtonEvent }
type ObstructionEvent struct{ Active bool }
type StopButton struct{ Active bool }
type DoorTimeout struct{}

func New() *Elevator {
	e := &Elevator{
		floor:     -1,
		doorTimer: time.NewTimer(3 * time.Second),
		orders:    make([]bool, 4),
		direction: Stop,
	}
	fmt.Println("Initial state: Init")
	e.fsm = fsm.NewFSM(e.initState)
	elevio.SetMotorDirection(elevio.MD_Down)
	return e
}

func (e *Elevator) addOrder(floor int) {
	e.orders[floor] = true
	elevio.SetButtonLamp(elevio.BT_Cab, floor, true)
}

func (e *Elevator) clearOrder(floor int) {
	e.orders[floor] = false
	elevio.SetButtonLamp(elevio.BT_Cab, floor, false)
}

func (e *Elevator) clearAllOrders() {
	for floor := range e.orders {
		e.clearOrder(floor)
	}
}

func (e *Elevator) getNextTarget() int {
	// Going up - find next order above current floor
	if e.direction == Up {
		for floor := e.floor + 1; floor < len(e.orders); floor++ {
			if e.orders[floor] {
				return floor
			}
		}
	}

	// Going down - find next order below current floor
	if e.direction == Down {
		for floor := e.floor - 1; floor >= 0; floor-- {
			if e.orders[floor] {
				return floor
			}
		}
	}

	// No directional orders, check if any orders
	for floor := len(e.orders) - 1; floor >= 0; floor-- {
		if e.orders[floor] {
			return floor
		}
	}

	return -1
}

func (e *Elevator) hasOrderAtFloor(floor int) bool {
	return e.orders[floor]
}

func (e *Elevator) initState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case FloorArrival:
		e.floor = evt.Floor
		e.direction = Stop
		elevio.SetFloorIndicator(evt.Floor)
		elevio.SetMotorDirection(elevio.MD_Stop)
		fmt.Println("State transition: init -> idleAtFloor")
		return e.idleAtFloorState
	}
	return e.initState
}

func (e *Elevator) idleAtFloorState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case ButtonPress:
		e.addOrder(evt.Event.Floor)
		if e.floor < evt.Event.Floor {
			e.direction = Up
			elevio.SetMotorDirection(elevio.MD_Up)
			fmt.Println("State transition: idle -> moving")
			return e.movingState
		}
		if e.floor > evt.Event.Floor {
			e.direction = Down
			elevio.SetMotorDirection(elevio.MD_Down)
			fmt.Println("State transition: idle -> moving")
			return e.movingState
		}
		if e.floor == evt.Event.Floor {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			e.clearOrder(evt.Event.Floor)
			e.doorTimer.Reset(doorOpenTimeSeconds)
			fmt.Println("State transition: idle -> door open")
			return e.doorOpenState
		}
	case StopButton:
		if evt.Active {
			e.clearAllOrders()
			e.direction = Stop
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			e.doorTimer.Reset(3 * time.Second)
			fmt.Println("State transition: idleAtFloor -> doorOpen")
			return e.doorOpenState
		}
	}
	return e.idleAtFloorState
}

func (e *Elevator) movingState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case FloorArrival:
		e.floor = evt.Floor
		elevio.SetFloorIndicator(evt.Floor)
		if e.hasOrderAtFloor(evt.Floor) {
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			e.clearOrder(evt.Floor)
			e.doorTimer.Reset(doorOpenTimeSeconds)
			fmt.Println("State transition: moving -> door open")
			return e.doorOpenState
		}
	case ButtonPress:
		e.addOrder(evt.Event.Floor)
	case StopButton:
		if evt.Active {
			e.clearAllOrders()
			elevio.SetMotorDirection(elevio.MD_Stop)
			fmt.Println("State transition: moving -> idleBetweenFloors")
			return e.idleBetweenFloorsState
		}
	}
	return e.movingState
}

func (e *Elevator) idleBetweenFloorsState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case ButtonPress:
		e.addOrder(evt.Event.Floor)
		if e.floor < evt.Event.Floor {
			e.direction = Up
			elevio.SetMotorDirection(elevio.MD_Up)
			fmt.Println("State transition: idle between floors -> moving")
			return e.movingState
		}
		if e.floor > evt.Event.Floor {
			e.direction = Down
			elevio.SetMotorDirection(elevio.MD_Down)
			fmt.Println("State transition: idle between floors -> moving")
			return e.movingState
		}
		if e.floor == evt.Event.Floor {
			if e.direction == Up {
				e.direction = Down
				elevio.SetMotorDirection(elevio.MD_Down)
				fmt.Println("State transition: idle between floors -> moving")
				return e.movingState
			}
			if e.direction == Down {
				e.direction = Up
				elevio.SetMotorDirection(elevio.MD_Up)
				fmt.Println("State transition: idle between floors -> moving")
				return e.movingState
			}
		}
	}
	return e.idleBetweenFloorsState
}

func (e *Elevator) doorOpenState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case ObstructionEvent:
		if evt.Active {
			e.doorTimer.Stop()
			fmt.Println("State transition: door open -> obstructed")
			return e.obstructedState
		}
	case DoorTimeout:
		elevio.SetDoorOpenLamp(false)
		nextFloor := e.getNextTarget()
		if nextFloor == -1 {
			e.direction = Stop
			fmt.Println("State transition: door open -> idleAtFloor")
			return e.idleAtFloorState
		} else if nextFloor > e.floor {
			e.direction = Up
			elevio.SetMotorDirection(elevio.MD_Up)
			fmt.Println("State transition: door open -> moving")
			return e.movingState
		} else if nextFloor < e.floor {
			e.direction = Down
			elevio.SetMotorDirection(elevio.MD_Down)
			fmt.Println("State transition: door open -> moving")
			return e.movingState
		}
	case ButtonPress:
		if e.floor == evt.Event.Floor {
			e.doorTimer.Reset(doorOpenTimeSeconds)
			fmt.Println("State transition: door open -> door open")
			return e.doorOpenState
		}
		e.addOrder(evt.Event.Floor)
	case StopButton:
		if evt.Active {
			e.clearAllOrders()
			e.direction = Stop
			elevio.SetMotorDirection(elevio.MD_Stop)
			elevio.SetDoorOpenLamp(true)
			e.doorTimer.Reset(doorOpenTimeSeconds)
			fmt.Println("State transition: door open -> door open")
			return e.doorOpenState
		}
	}
	return e.doorOpenState
}

func (e *Elevator) obstructedState(event interface{}) fsm.StateFunc {
	switch evt := event.(type) {
	case ObstructionEvent:
		if !evt.Active {
			e.doorTimer.Reset(doorOpenTimeSeconds)
			fmt.Println("State transition: obstructed -> door open")
			return e.doorOpenState
		}
	case ButtonPress:
		e.addOrder(evt.Event.Floor)
	}
	return e.obstructedState
}

func (e *Elevator) Run(ctx context.Context) {
	buttonEvents := make(chan elevio.ButtonEvent)
	floorSensor := make(chan int)
	obstructionSwitch := make(chan bool)
	stopButton := make(chan bool)

	go elevio.PollButtons(buttonEvents)
	go elevio.PollFloorSensor(floorSensor)
	go elevio.PollObstructionSwitch(obstructionSwitch)
	go elevio.PollStopButton(stopButton)

	e.fsm.Start(ctx)

	for {
		select {
		case button := <-buttonEvents:
			e.fsm.Send(ButtonPress{button})
		case floor := <-floorSensor:
			e.fsm.Send(FloorArrival{floor})
		case obstruction := <-obstructionSwitch:
			e.fsm.Send(ObstructionEvent{obstruction})
		case stop := <-stopButton:
			e.fsm.Send(StopButton{stop})
		case <-e.doorTimer.C:
			e.fsm.Send(DoorTimeout{})
		case <-ctx.Done():
			return
		}
	}
}
