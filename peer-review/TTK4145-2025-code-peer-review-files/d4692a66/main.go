package main

import (
	"G19_heis2/Heis/FSM"
	"G19_heis2/Heis/communication"
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/driver/elevio"
	"G19_heis2/Heis/logic"
	"fmt"
)

func main() {
	numFloors := 4

	elevio.Init("localhost:15658", numFloors) //elevatorserver --port "15658"  -  simelevatorserver --port "15657"  -  go run main.go
	id := config.InitID()
	elevator := config.InitElev(id)

	txHeartbeat := make(chan communication.HeartBeat)
	rxHeartbeat := make(chan communication.HeartBeat)

	communication.StartHeartBeat(&elevator, txHeartbeat, rxHeartbeat)

	StateRX := make(chan *config.Elevator)
	StateTX := make(chan *config.Elevator)

	channels := config.NetworkChannels{
		StateRX: StateRX,
		StateTX: StateTX,
	}

	communication.StartStateUpdate(&elevator, channels)

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	fmt.Println("Elevator system initialized...")
	go logic.RunHRA(&elevator, &config.GlobalState)
	//go logic.SetToUnconfirmed(&config.GlobalState, &elevator)

	go FSM.Fsm(&elevator, drv_buttons, drv_obstr, drv_stop, drv_floors, numFloors, channels, &config.GlobalState)
	select {}
}
