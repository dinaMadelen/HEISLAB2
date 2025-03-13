//go:build testmode

package main

import (
	"fmt"
	"log"
	"realtime_systems/udp_heartbeat"
	"strconv"
	"time"

	"github.com/joho/godotenv"
)

func main() {
	envFile, err := godotenv.Read("../.env")
	if err != nil {
		log.Fatal("Error loading .env file")
		return
	}

	subnet_ip := envFile["SUBNET"]
	broadcast_port := envFile["ELEVATOR_ALIVE_BROADCAST_PORT"]
	own_id := envFile["ELEVATOR_ID"]
	alive_frequency, err := strconv.Atoi(envFile["ALIVE_FREQUENCY"])
	if err != nil {
		log.Fatal("Error converting ALIVE_FREQUENCY to int")
		return
	}
	n_elevators, err := strconv.Atoi(envFile["NUMBER_OF_ELEVATORS"])
	if err != nil {
		log.Fatal("Error converting NUMBER_OF_ELEVATORS to int")
		return
	}
	alive_watchdog_timeout, err := strconv.Atoi(envFile["ALIVE_WATCHDOG_TIMEOUT"])
	if err != nil {
		log.Fatal("Error converting ALIVE_WATCHDOG_TIMEOUT to int")
		return
	}

	// Example code
	NodeStatus := udp_heartbeat.NodeStatus{
		Alive_nodes: make(map[string]bool),
	}

	// Initialize the map of nodes which should be present in network
	for i := 0; i < n_elevators; i++ {
		NodeStatus.Alive_nodes[strconv.Itoa(i)] = false
	}

	go udp_heartbeat.Broadcast_alive(subnet_ip, broadcast_port, own_id, alive_frequency)
	go udp_heartbeat.Listen_broadcast(subnet_ip, broadcast_port, own_id)
	go udp_heartbeat.TrackNodeStatus(&NodeStatus, time.Duration(alive_watchdog_timeout)*time.Second)

	go func() {
		for {
			fmt.Println("---Node status---")
			NodeStatus.Mu_an.Lock()
			for id, active := range NodeStatus.Alive_nodes {
				fmt.Printf("%s : %t ", id, active)
			}
			NodeStatus.Mu_an.Unlock()
			fmt.Println("")
			time.Sleep(1 * time.Second)
		}
	}()

	select {}

}
