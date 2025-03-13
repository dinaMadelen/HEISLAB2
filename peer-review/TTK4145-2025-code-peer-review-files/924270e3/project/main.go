package main

import (
	"os"
	"fmt"
	"project/sanntid/network/id"
	"project/sanntid/network/bcast"
	"project/sanntid/network/peers"
	"project/sanntid/elevator/elevio"
	req "project/sanntid/requests/request"
	"project/sanntid/elevator/elevator"
	"project/sanntid/config"
)

func main() {
	// Assign unique ID to machine
	ID, _ := id.AssignId()
	fmt.Println("Your ID: ", ID)

	// Read configuration file
	conf := config.ReadConfig(os.Args[1])
	port := conf.Port
	//numFloors := conf.NumFloors

	// Peers overview
	transmitEnable := make(chan bool)
	peerUpdateCh := make(chan peers.PeerUpdate)

	go peers.Transmitter(8000, ID, transmitEnable)
	go peers.Receiver(8000, peerUpdateCh)

	// Initialize elevator io
	elevAddr := fmt.Sprintf("127.0.0.1:%d", port)
	fmt.Println(elevAddr)
	//elevAddr := "127.0.0.1:15657"
	elevio.Init(elevAddr, 4)

	// Transmit channels
	hallReqTx := make(chan req.HallRequestTransmit)
	cabReqTx := make(chan req.CabRequestTransmit)
	reqAssigner := make(chan req.Request)

	// Receive channels
	hallReqRx := make(chan req.HallRequestTransmit)
	cabReqRx := make(chan req.CabRequestTransmit)
	reqUpdate := make(chan req.Request)

	// Broadcast HallRequests
	go bcast.Transmitter(15697, reqTx)
	go bcast.Receiver(15697, reqRx)

	// Broadcast CabRequests
	//go bcast.Transmitter(15697, reqTx)
	//go bcast.Receiver(15697, reqRx)

	// Gorutine for handling requests
	go req.RequestsHandler(ID, reqTx, reqRx, reqAssigner, reqUpdate, peerUpdateCh)

	// Gorutine for handling tasks
	elev := elevator.InitState(ID, 4)
	go elev.StateMachine(reqAssigner, reqUpdate)

	for {
	}
}
