package main

import (
	"encoding/json"
	"log"
	"net"
	"os"

	"group48.ttk4145.ntnu/elevators/comms"
	"group48.ttk4145.ntnu/elevators/elevatordriver"
	"group48.ttk4145.ntnu/elevators/elevatorio"
	"group48.ttk4145.ntnu/elevators/healthmonitor"
	"group48.ttk4145.ntnu/elevators/models"
	"group48.ttk4145.ntnu/elevators/orderserver"
	"group48.ttk4145.ntnu/elevators/requests"
)

func main() {
	config, err := LoadConfig("config.json")
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Elevator IO module initialization
	var unvalidatedRequests = make(chan models.RequestMessage)
	var floorSensorUpdates = make(chan int)
	var obstructionSwitchUpdates = make(chan bool)

	elevatorio.Init(config.ElevatorAddr, config.NumFloors, models.Id(config.LocalPeerId))
	go elevatorio.PollRequests(unvalidatedRequests)
	go elevatorio.PollFloorSensor(floorSensorUpdates)
	go elevatorio.PollObstructionSwitch(obstructionSwitchUpdates)

	// Elevator Driver module initialization
	var orders = make(chan models.Orders)
	var internalElevatorStateToComms = make(chan models.ElevatorState)
	var elevatorStatesToOrders = make(chan models.ElevatorState)
	var internalElevatorState = make([]chan<- models.ElevatorState, 2)
	internalElevatorState[0] = internalElevatorStateToComms
	internalElevatorState[1] = elevatorStatesToOrders
	go elevatordriver.Starter(
		obstructionSwitchUpdates,
		floorSensorUpdates,
		orders,
		unvalidatedRequests,
		internalElevatorState,
		models.Id(config.LocalPeerId))

	// Order module initialization
	var aliveStatus = make(chan []models.Id)
	var validatedRequestsToOrder = make(chan models.Request)
	go orderserver.RunOrderServer(
		validatedRequestsToOrder,
		elevatorStatesToOrders,
		aliveStatus,
		orders,
		models.Id(config.LocalPeerId))

	// Health monitor module initialization
	var ping = make(chan models.Id)
	go healthmonitor.RunMonitor(ping, aliveStatus, models.Id(config.LocalPeerId))

	// Comms module initialization
	var internalValidatedRequestsToComms = make(chan models.Request)

	go comms.RunComms(
		models.Id(config.LocalPeerId),
		net.IPAddr{IP: net.ParseIP(config.LocalAddr)},
		config.LocalPort,
		internalElevatorStateToComms,
		internalValidatedRequestsToComms,
		elevatorStatesToOrders,
		unvalidatedRequests,
		ping)

	// Request module initialization
	var validatedRequests = make([]chan<- models.Request, 2)
	validatedRequests[0] = validatedRequestsToOrder
	validatedRequests[1] = internalValidatedRequestsToComms
	go requests.RunRequestServer(
		unvalidatedRequests,
		aliveStatus,
		validatedRequests)

	select {}
}

type Config struct {
	ElevatorAddr string `json:"elevator_addr"`
	NumFloors    int    `json:"num_floors"`
	LocalPeerId  int    `json:"local_peer_id"`
	LocalAddr    string `json:"local_addr"`
	LocalPort    uint16 `json:"local_port"`
}

func LoadConfig(filename string) (*Config, error) {
	file, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	decoder := json.NewDecoder(file)
	config := &Config{}
	err = decoder.Decode(config)
	if err != nil {
		return nil, err
	}

	return config, nil
}
