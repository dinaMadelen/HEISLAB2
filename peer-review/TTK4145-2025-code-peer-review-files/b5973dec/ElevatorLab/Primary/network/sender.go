package network

import (
	"encoding/json"
	"log"
	"net"
	"time"
)

// --------------- SENDER FUNCTIONS --------------- //

// --- GLOBAL FUNCTIONS --- //

func SendElevatorState(port string, ip string, state interface{}) {
	for {
		conn, err := net.Dial("tcp", ip+":"+port)
		if err != nil {
			log.Println("Failed to connect to primary:", err)
			time.Sleep(1 * time.Second)
			continue
		}
		defer conn.Close()

		data, err := json.Marshal(state)
		if err != nil {
			log.Println("Failed to encode state:", err)
			return
		}

		_, err = conn.Write(data)
		if err != nil {
			log.Println("Failed to send state update:", err)
		}
		return
	}
}

func SendHeartBeat(port string, ip string) {
	beat := "Alive"
	for {
		conn, err := net.Dial("tcp", ip+":"+port)
		if err != nil {
			log.Println("Failed to connect to primary:", err)
			time.Sleep(1 * time.Second)
			continue
		}

		data, err := json.Marshal(beat)
		if err != nil {
			log.Println("Failed to encode heartbeat:", err)
			conn.Close()
			continue
		}

		_, err = conn.Write(data)
		if err != nil {
			log.Println("Failed to send heartbeat update:", err)
		}
		conn.Close()
		time.Sleep(1 * time.Second)
	}
}
