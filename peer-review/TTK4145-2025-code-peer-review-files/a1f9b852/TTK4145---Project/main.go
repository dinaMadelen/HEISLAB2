package main

import (
	network "TTK4145---project/Network-go"
	config "TTK4145---project/config"
	driver "TTK4145---project/driver-go"
	elevio "TTK4145---project/driver-go/elevio"
	"flag"
)

func main() {

	go network.Network(&config.ElevatorInstance)
	go driver.RunElevator()

	select {}
}

func init() {
	flag.StringVar(&config.ElevatorInstance.ID, "id", "", "id of this peer")
	flag.StringVar(&config.Port, "port", "15657", "port to listen on")
	flag.Parse()

	config.ElevatorInstance = config.Elevator{
		ID:        config.ElevatorInstance.ID,
		State:     config.Idle,
		Direction: elevio.MD_Stop,
		Floor:     0,
		Queue:     [config.NumFloors][config.NumButtons]config.OrderState{},
	}

	config.Elevators = make(map[string]config.Elevator)
	config.Elevators[config.ElevatorInstance.ID] = config.ElevatorInstance
}
