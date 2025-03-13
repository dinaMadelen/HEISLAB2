package node

import (
	"elevatorproject/Network-go/network/bcast"
	elevatoralgorithm "elevatorproject/driver-go/elevator_algorithm"
	"fmt"
	"time"
)

// Node ID
// Sending state from elevator
// detect disconnect via timeout

//

const (
	port = 20050
)

type HeartBeat struct {
	ID       int
	Msg      string
	Elevator elevatoralgorithm.Elevator
}

type Node struct {
	state *elevatoralgorithm.Elevator
	msgRx chan HeartBeat
	msgTx chan HeartBeat
	ID    int
}

func NodeInit(elevator *elevatoralgorithm.Elevator) *Node {
	var n Node

	n.state = elevator
	n.msgRx = make(chan HeartBeat)
	n.msgTx = make(chan HeartBeat)

	go bcast.Transmitter(port, n.msgTx)
	go bcast.Receiver(port, n.msgRx)

	// heartBeat := HeartBeat{
	// 	Msg:      "Hello",
	// 	Elevator: *elevator,
	// }

	go n.receive()
	go n.transmit()

	return &n
}

func (n *Node) receive() {
	for {
		msg := <-n.msgRx
		fmt.Println(msg.Msg)
		fmt.Println("etasjen heisen er i :")
		fmt.Println(msg.Elevator.GetFloor())
	}
}

func (n *Node) transmit() {
	for {
		heartBeat := HeartBeat{
			Msg:      "Hello",
			Elevator: *n.state,
		}

		n.msgTx <- heartBeat
		fmt.Println(heartBeat.Elevator.GetFloor())
		time.Sleep(1 * time.Second)
	}
}

func readLifesignals() {
	// for peer {
	//  if alreadyDiscovered, set lastSeen as now()
	//}
}

func timeout() {
	// for peer {
	//  if peer.lastSeen is more than five seconds ago {
	//    removePeer()
	//}
	// }
}
