package main

// Hub is selected to be the computer with the lowest IP address.

// Hub is the central point of communication between the different modules of the system -> All communication goes through the hub.

// Hub must appoint a backup hub in case of failure. The backup hub must be the computer with the second lowest IP address.
// The backup hub must be able to take over the role of the hub within 2 seconds of the hub failing.

// Elevators broadcast their world view to the hub, and it the backup hub must have the same world view as the hub for it to distribute and order.

// The hub must be able to handle the following messages:
// - Order from the order module
// - Elevator state from elevator(s)
// - Elevator state from the backup hub

import (
	"fmt"
	"hub/hub_network"
	"time"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub_fsm"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hub_network"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/network"
)

// Hub main function

func main() {
	fmt.Println("Hub started!")

	// Get local IP
	localIP, err := hub_network.GetLocalIP()
	localIPStr := localIP.String()
    if err != nil {
        fmt.Println("Error obtaining local IP:", err)
        return
    }
    fmt.Println("Local IP:", localIP)

	worldViewChannel := make(chan hub.WorldView)
	pendingChannel := make(chan hub.ButtonEvent)
	heartbeatChannel := make(chan hub.Heartbeat)

	hub_fsm.Init(localIP) // Find state

	go hub_network.ReceivePackets(worldViewChannel, pendingChannel, heartbeatChannel) // Elevators should send at fixed rate her (20 Hz or something)

	// Goroutine to send heartbeats to backup hub
	go func() {
		for {
			// Broadcast heartbeat to backup hub (and other elevators) (Port 20017)
			// If no response within 2 seconds, backup hub takes over
			hub_network.SendHeartbeat(time.Now().Unix(), hub.Heartbeat{IP: localIPStr, State: hub_fsm.GetState(), Instruction: -1})
			time.Sleep(200 * time.Millisecond)
		}
	}()

	// Initalize backup

	for {
		select {
		case BtnPress := <-pendingChannel:
			fmt.Printf("%+v\n", BtnPress)
			hub_fsm.OnReceivedOrder(BtnPress)
		case worldView <-worldViewChannel:
			fmt.Println("Hub: Received world view")
			hub_fsm.OnUpdateWorldView(worldView)
		case heartbeat <-heartbeatChannel:
			fmt.Println("Hub: Received heartbeat")
			hub_fsm.OnHeartbeat(heartbeat)
	}
	}
}