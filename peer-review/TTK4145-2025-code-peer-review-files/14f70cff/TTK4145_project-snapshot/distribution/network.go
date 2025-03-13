package distribution

import (
	"elevator/elevator_interface"
	"fmt"
	"net"
	"os"
	"strings"
)

var id string

func GetPersonalID() string {
	if id == "" {
		id = fmt.Sprintf("peer-%s-%d", getLocalIP(), os.Getpid())
	}
	return id
}

func getLocalIP() string {
	var localIP string
	conn, _ := net.DialTCP("tcp4", nil, &net.TCPAddr{IP: []byte{8, 8, 8, 8}, Port: 53})
	localIP = strings.Split(conn.LocalAddr().String(), ":")[0]
	return localIP
}

func RunNetwork() {
	id = GetPersonalID()

	peerUpdateCh := make(chan PeerUpdate)
	peerTxEnable := make(chan bool)

	worldviewTx := make(chan elevator_interface.Worldview)
	worldviewRx := make(chan elevator_interface.Worldview)

	go peerTransmitter(15647, id, peerTxEnable)
	go peerReceiver(15647, peerUpdateCh)
	go Transmitter(16569, worldviewTx)
	go Receiver(16569, worldviewRx)

	go elevator_interface.SendWorldview(worldviewTx)

	fmt.Println("Started")
}
