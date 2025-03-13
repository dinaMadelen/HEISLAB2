//package network_master
package main

import (
	"Network-go/network/bcast"
	"Network-go/network/localip"
	"Network-go/network/peers"
	"main/utilities"
	"main/elev_algo_go/elevator"
	"flag"
	"fmt"
	"os"
	"time"
)

// Node message used by the master containing the distribution message.
type Node_msg struct {
	Dist_msg utilities.OrderDistributionMessage
}

// For testing purposes
func main() {
	// Make two channels, one called assign_chan to receive the orders to distribute, received from the hall-order assigner. The other channel, bcast_sorders_chan, is used by the master to send these orders to the slaves (including itself).
	assign_chan := make(chan utilities.OrderDistributionMessage)

	// Channel to be passed into node_tx for broadcasting. Has not been implemented yet.
	bcast_sorders_chan := make(chan utilities.OrderDistributionMessage)

	// Also need to implement a channel for master to send request orders to the assigner.
	// t_rorders_chan := make(chan utilities.OrderDistributionMessage

	go network_master(assign_chan, bcast_sorders_chan)
	// Simulate the master receiving orderlines from the hall-order-assigner.
	for {	
		assign_chan <- utilities.OrderDistributionMessage{Label : "Ø", Orderlines : [3][elevator.N_FLOORS][elevator.N_BUTTONS-1]bool{
			{	{true,false},
				{false,true},
				{true,true},
				{false,false},
			},
			{	{true,false},
				{false,true},
				{true,true},
				{false,false},
			},
			{	{true,false},
				{false,true},
				{true,true},
				{false,false},
			},
		}	}
	
	}
}

func network_master(assign_chan <-chan utilities.OrderDistributionMessage, bcast_sorders_chan chan<- utilities.OrderDistributionMessage) {
	var id string
	flag.StringVar(&id, "id", "", "id of this peer")
	flag.Parse()

	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		id = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
	}

	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	go peers.Transmitter(15647, id, peerTxEnable)
	go peers.Receiver(15647, peerUpdateCh)

	node_tx := make(chan Node_msg)
	node_rx := make(chan Node_msg)
	go bcast.Transmitter(16569, node_tx)
	go bcast.Receiver(16569, node_rx)

	node_msg := Node_msg{}

	go func() {
		for {
			select {
			// Update the distribution if the hall assigner sends an updated list
			case assign := <-assign_chan:
				node_msg.Dist_msg = assign
			default:
			}
			node_tx <- node_msg
			time.Sleep(1 * time.Second)

			// Broadcast service orders to the controllers
			// bcast_sorders_chan <- a.Dist_msg	
		}
	}()

	for {
		select {
		case p := <-peerUpdateCh:
			fmt.Printf("Peer update:\n")
			fmt.Printf("  Peers:    %q\n", p.Peers)
			fmt.Printf("  New:      %q\n", p.New)
			fmt.Printf("  Lost:     %q\n", p.Lost)

		case a := <- node_rx:
			fmt.Printf("Received: %v\n", a)
		}
	}
}
