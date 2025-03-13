package driver

import (
	"TTK4145---project/config"
	"TTK4145---project/driver-go/elevio"
	"fmt"
)

func RunElevator() {

	numFloors := config.NumFloors

	elevio.Init("localhost:"+config.Port, numFloors) // default 15657

	var d elevio.MotorDirection = elevio.MD_Stop

	//elevio.SetMotorDirection(d)

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	for {
		select {
		case a := <-drv_buttons:
			fmt.Printf("%+v\n", a)
			if a.Button == elevio.BT_Cab {
				UpdateQueue(a.Floor, int(config.ButtonCab), config.Confirmed)
				elevio.SetButtonLamp(a.Button, a.Floor, true)
			} else {
				UpdateQueue(a.Floor, int(a.Button), config.Confirmed)
			}

			decideDir()

		case a := <-drv_floors:
			config.ElevatorInstance.Floor = a
			fmt.Printf("%+v\n", a)

			decideDir()

		case a := <-drv_obstr:
			fmt.Printf("%+v\n", a)
			if a {
				elevio.SetMotorDirection(elevio.MD_Stop)
				config.ElevatorInstance.Direction = elevio.MD_Stop
			} else {
				elevio.SetMotorDirection(d)
				config.ElevatorInstance.Direction = d
			}

		case a := <-drv_stop:
			fmt.Printf("%+v\n", a)
			for f := 0; f < numFloors; f++ {
				for b := elevio.ButtonType(0); b < 3; b++ {
					elevio.SetButtonLamp(b, f, false)
				}
			}
		case <-config.MyQueue:
			decideDir()

		}

	}
}
