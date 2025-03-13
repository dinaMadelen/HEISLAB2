package network

import (
	"TTK4145---project/Network-go/network/bcast"
	"TTK4145---project/Network-go/network/localip"
	"TTK4145---project/Network-go/network/peers"
	"TTK4145---project/config"
	"TTK4145---project/driver-go"
	"TTK4145---project/driver-go/elevio"
	"fmt"
	"os"
	"time"
)

// We define some custom struct to send over the network.
// Note that all members we want to transmit must be public. Any private members
//
//	will be received as zero-values.

func Network(elevatorInstance *config.Elevator) {
	// Our id can be anything. Here we pass it on the command line, using
	//  `go run main.go -id=our_id`

	var id = elevatorInstance.ID

	// ... or alternatively, we can use the local IP address.
	// (But since we can run multiple programs on the same PC, we also append the
	//  process ID)
	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		id = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
	}

	// We make a channel for receiving updates on the id's of the peers that are
	//  alive on the network
	peerUpdateCh := make(chan peers.PeerUpdate)
	// We can disable/enable the transmitter after it has been started.
	// This could be used to signal that we are somehow "unavailable".
	peerTxEnable := make(chan bool)
	go peers.Transmitter(15647, id, peerTxEnable)
	go peers.Receiver(15647, peerUpdateCh)

	// We make channels for sending and receiving our custom data types
	elevatorTx := make(chan config.Elevator) // Transmitter
	elevatorRx := make(chan config.Elevator) // Receiver
	// ... and start the transmitter/receiver pair on some port
	// These functions can take any number of channels! It is also possible to
	//  start multiple transmitters/receivers on the same port.
	go bcast.Transmitter(16569, elevatorTx)
	go bcast.Receiver(16569, elevatorRx)

	// The example message. We just send one of these every second.
	go func() {
		for {
			elevatorTx <- *elevatorInstance
			time.Sleep(20 * time.Millisecond)
		}
	}()

	fmt.Println("Started")
	for {
		select {
		case p := <-peerUpdateCh:
			fmt.Printf("Peer update:\n")
			fmt.Printf("  Peers:    %q\n", p.Peers)
			fmt.Printf("  New:      %q\n", p.New)
			fmt.Printf("  Lost:     %q\n", p.Lost)

			for _, lostPeer := range p.Lost {
				delete(config.Elevators, lostPeer)
			}

		case a := <-elevatorRx:
			fmt.Printf("Received: %#v\n", a)

			elev := config.Elevator{
				ID:        a.ID,
				State:     a.State,
				Direction: a.Direction,
				Floor:     a.Floor,
				Queue:     a.Queue,
			}

			config.Elevators[a.ID] = elev

			SyncHallRequests()

		}
	}
}

func SyncHallRequests() {

	// if all elevators have the same unconfirmed request, make the request confirmed
	for i := 0; i < config.NumFloors; i++ {
		isConfirmedUp := true
		for _, elev := range config.Elevators {
			if elev.Queue[i][config.ButtonUp] != config.Unconfirmed {
				isConfirmedUp = false
				break
			}
		}
		if isConfirmedUp {
			driver.UpdateQueue(i, int(config.ButtonUp), config.Confirmed)
			elevio.SetButtonLamp(elevio.BT_HallUp, i, true)
		}

		isConfirmedDown := true
		for _, elev := range config.Elevators {
			if elev.Queue[i][config.ButtonDown] != config.Unconfirmed {
				isConfirmedDown = false
				break
			}
		}
		if isConfirmedDown {

			driver.UpdateQueue(i, int(config.ButtonDown), config.Confirmed)
			elevio.SetButtonLamp(elevio.BT_HallDown, i, true)
		}
	}

	// if one elevator is one step ahead, make the request the same as the one step ahead
	for i := 0; i < config.NumFloors; i++ {
		for _, elev := range config.Elevators {
			up := elev.Queue[i][config.ButtonUp] - config.ElevatorInstance.Queue[i][config.ButtonUp]
			down := elev.Queue[i][config.ButtonDown] - config.ElevatorInstance.Queue[i][config.ButtonDown]

			if up == 1 || up == -2 {
				driver.UpdateQueue(i, int(config.ButtonUp), elev.Queue[i][config.ButtonUp])
			}

			if down == 1 || down == -2 {
				driver.UpdateQueue(i, int(config.ButtonDown), elev.Queue[i][config.ButtonDown])
			}
		}
	}

}
