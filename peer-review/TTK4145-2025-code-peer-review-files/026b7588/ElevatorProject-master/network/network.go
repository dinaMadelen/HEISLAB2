package network

import (
	"ElevatorProject/elevio"
	"ElevatorProject/network/bcast"
	"ElevatorProject/network/localip"
	"ElevatorProject/network/node"
	"flag"
	"fmt"
	"os"
)

type OrderMsg struct {
	Floor  int
	Button elevio.ButtonType
	Active bool
}


func Network(orderTx chan OrderMsg, orderRx chan OrderMsg) string {
	var id string
	flag.StringVar(&id, "id", "", "id of this node")
	flag.Parse()

	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		id = fmt.Sprintf("node-%s-%d", localIP, os.Getpid())
	}

	nodeUpdateCh := make(chan node.NodeUpdate)
	nodeTxEnable := make(chan bool)

	go node.Transmitter(15500, id, nodeTxEnable)
	go node.Receiver(15500, nodeUpdateCh)

	go bcast.Transmitter(16582, orderTx)
	go bcast.Receiver(16582, orderRx)

	for {
		select {
		case node := <-nodeUpdateCh:
			fmt.Printf("Node update:\n")
			fmt.Printf("  Master:    %q\n", node.Master)
			fmt.Printf("  Slaves:    %q\n", node.Slaves)
			fmt.Printf("  New:      %q\n", node.New)
			fmt.Printf("  Lost:     %q\n", node.Lost)

		case a := <-orderRx:
			fmt.Printf("Received: %#v\n", a.Active)
			fmt.Printf("Button: %#v\n", a.Button)
			fmt.Printf("Floor: %#v\n", a.Floor)
		}
	}
	return id
}