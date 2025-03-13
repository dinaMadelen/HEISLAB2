package initialize

import (
	"flag"
	"fmt"
	"os"
	"strconv"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
	"github.com/Eirik-a-Johansen/trippel_elevator/network/localip"
)

/*
This module should define the elevator to a known state when the program is started
*/

func Init(e *elevator.Elevator) {
	e.Functional = false //set to not functional until init is complete

	driver.Init("localhost:50010", driver.N_Floors)

	//Drive to valid floor
	currentFloor := driver.GetFloor()
	if currentFloor == -1 {
		driver.SetMotorDirection(-1)
		for currentFloor == -1 {
			currentFloor = driver.GetFloor()
			if currentFloor != -1 {
				driver.SetMotorDirection(0)
			}
		}
	}

	//sets all order buttons to off
	for i := 0; i < driver.N_Floors; i++ {
		for j := 0; j < driver.N_Buttons; j++ {
			driver.SetButtonLamp(driver.ButtonType(j), i, false)
		}
	}

	driver.SetFloorIndicator(driver.GetFloor())

	driver.SetStopLamp(false)
	driver.SetDoorOpenLamp(false)

	//set all orders to undelegated
	for i := range elevator.Delegated {
		for j := range elevator.Delegated[i] {
			elevator.Delegated[i][j] = -1
		}
	}

	e.Floor = driver.GetFloor()
	e.Dirn = elevator.D_Stop
	e.Behaviour = elevator.EB_Idle
	e.OnFloor = true
	e.OpenDoor = false
	e.DoorObstruction = false
	e.Stop = false
	e.IsMaster = true

	//Set elevator id based on id given when starting program: -id=x format
	var id string
	flag.StringVar(&id, "id", "", "id of this peer")
	flag.Parse()
	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "Disconnected"
		}
		id = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
	}

	elevator.LocalElevator.ID, _ = strconv.Atoi(id)
	elevator.Elevators[elevator.LocalElevator.ID] = elevator.LocalElevator

	e.Functional = true //elevator is functional when init is complete
}
