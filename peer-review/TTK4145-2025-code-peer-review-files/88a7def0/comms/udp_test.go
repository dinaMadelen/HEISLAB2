package comms

import (
	"net"
	"testing"

	m "group48.ttk4145.ntnu/elevators/models"
)

var msg = udpMessage{
	Source: 1,
	EState: m.ElevatorState{
		Id:        1,
		Direction: m.Up,
		Floor:     2,
		Behavior:  m.Moving,
	},
	Requests: []m.Request{
		{Origin: m.Origin{Source: m.Hall{}, Floor: 1, ButtonType: m.HallUp}, Status: m.Unknown},
	},
}

func TestCoding(t *testing.T) {
	encoded := encode(msg)
	decoded := decode(encoded)

	if !isMsgEqual(msg, decoded) {
		t.Errorf("Expected %v, got %v", msg, decoded)
	}
}

func TestUdp(t *testing.T) {
	var send = make(chan udpMessage)
	var receive = make(chan udpMessage)
	var local = net.UDPAddr{IP: net.ParseIP("localhost"), Port: 12345}

	go RunUdpReader(receive, local)
	go RunUdpWriter(send, local)

	send <- msg
	decoded := <-receive

	if !isMsgEqual(msg, decoded) {
		t.Errorf("Expected %v, got %v", msg, decoded)
	}
}

func isMsgEqual(msg1, msg2 udpMessage) bool {
	return msg1.Source == msg2.Source && msg1.EState == msg2.EState && len(msg1.Requests) == len(msg2.Requests)
}
