//go:build testmode

package main

import (
	"fmt"
	"log"
	commontypes "realtime_systems/common_types"
	"realtime_systems/queue_assigner"
	"realtime_systems/udp_heartbeat"
	"strconv"

	"github.com/joho/godotenv"
)

func main() {
	envFile, err := godotenv.Read("../.env")
	if err != nil {
		log.Fatal("Error loading .env file")
		return
	}

	subnet_ip := envFile["SUBNET"]
	alive_broadcast_port := envFile["ELEVATOR_ALIVE_BROADCAST_PORT"]
	status_broadcast_port := envFile["ELEVATOR_STATUS_BROADCAST_PORT"]
	own_id := envFile["ELEVATOR_ID"]
	alive_frequency, err := strconv.Atoi(envFile["ALIVE_FREQUENCY"])
	if err != nil {
		log.Fatal("Error converting ALIVE_FREQUENCY to int")
		return
	}
	n_elevators, err := strconv.Atoi(envFile["NUMBER_OF_ELEVATORS"])
	if err != nil {
		log.Fatal("Error converting NUMBER_OF_ELEVATORS to int")
	}

	_ = subnet_ip
	_ = alive_broadcast_port
	_ = status_broadcast_port
	_ = own_id
	_ = alive_frequency
	_ = n_elevators

	hallRequests := [][2]bool{{false, false}, {true, false}, {false, false}, {false, true}}

	elevatorStates := commontypes.HRAInput{
		HallRequests: hallRequests,
		States: map[string]commontypes.ElevState{
			"0": commontypes.ElevState{
				Behavior:    "moving",
				Floor:       2,
				Direction:   "up",
				CabRequests: []bool{false, false, false, true},
			},
			"1": commontypes.ElevState{
				Behavior:    "idle",
				Floor:       0,
				Direction:   "stop",
				CabRequests: []bool{false, false, false, false},
			},
		},
	}

	aliveNodes := udp_heartbeat.NodeStatus{
		Alive_nodes: map[string]bool{
			"0": true,
			"1": true,
		},
	}

	// Calculate the queue
	output := queue_assigner.StateToQueue(elevatorStates, &aliveNodes)

	// Print the output
	fmt.Printf("output: \n")
	for k, v := range output {
		fmt.Printf("%6v :  %+v\n", k, v)
	}

}
