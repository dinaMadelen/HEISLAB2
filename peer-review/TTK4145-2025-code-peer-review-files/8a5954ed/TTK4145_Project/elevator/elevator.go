package elevator

import (
	"color"
	"config"
	"elevio"
	"fmt"
	"net"
	"network"
	"os"
	"time"
)

var State = struct {
	Undefined, Idle, Moving, FloorStop, FullStop, DoorOpen string
}{
	"undefined", "idle", "moving", "floor_stop", "full_stop", "door_open",
}

type Elevator struct {
	Current_state  string
	Current_floor  int
	Target_floor   int
	Direction      elevio.MotorDirection
	Obstruction    bool
	Queue          *Queue
	Finished_queue *Queue
	ID             string
	Conn           *net.TCPConn
	Connected      bool

	//Timers
	Door_timer      *time.Timer
	Door_timer_done bool
	Reconnect_timer *time.Timer
}

func Create_elevator(id string, capacity int) *Elevator {
	return &Elevator{
		Current_state:   State.Undefined,
		Queue:           Create_queue(capacity),
		Finished_queue:  Create_queue(capacity),
		ID:              id,
		Door_timer_done: false,
		Door_timer:      time.NewTimer(config.Door_open_duration),
		Connected:       false,
		Reconnect_timer: time.NewTimer(config.Reconnect_delay),
	}
}

func Run_single_elevator() {
	id := os.Args[1]
	elevator := Create_elevator(id, config.Num_floors*3)
	elevator.Door_timer.Stop()

	elevio.Init("localhost:15657", config.Num_floors)
	Reset_all_lamps(config.Num_floors)
	elevio.SetDoorOpenLamp(false)

	drv_buttons := make(chan elevio.ButtonEvent, config.Num_floors*3)
	drv_floors := make(chan int)
	drv_obstruction := make(chan bool)
	drv_stop := make(chan bool)

	srv_listener := make(chan network.Message, config.Num_floors*3)
	srv_connLoss := make(chan *net.TCPConn)

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstruction)
	go elevio.PollStopButton(drv_stop)

	go elevator.Connect_elevator(srv_listener, srv_connLoss)
	go elevator.State_machine()

	fmt.Printf(color.Green+"Elevator %s ready for use. \n"+color.Reset, elevator.ID)

	for {
		select {
		case floor := <-drv_floors:
			elevator.Handle_floor_signal(floor)

		case btn := <-drv_buttons:
			elevator.Handle_button_signal(btn)

		case <-drv_stop:
			elevator.Current_state = State.FullStop

		case obs := <-drv_obstruction:
			elevator.Handle_obstruction_signal(obs)

		case <-elevator.Door_timer.C:
			elevator.Door_timer_done = true

		case <-srv_connLoss:
			elevator.Handle_connection_loss()

		case message := <-srv_listener:
			elevator.Handle_new_message(message)
		}
	}
}