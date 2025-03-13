package comms

import (
	"bytes"
	"encoding/gob"
	"log"
	"net"

	"group48.ttk4145.ntnu/elevators/models"
)

// udpMessage is the message type used for communication over UDP
type udpMessage struct {
	Source   models.Id
	EState   models.ElevatorState
	Requests []models.Request
}

// RunUdpReader listens for incoming UDP messages and sends them to the receiver channel
func RunUdpReader(
	receiver chan<- udpMessage,
	local net.UDPAddr) {
	// Setup
	var conn, err = net.ListenPacket("udp", local.String())
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	// Run the UDP reader
	for {
		var buf = make([]byte, 1024)
		_, _, err = conn.ReadFrom(buf)
		if err != nil {
			log.Println(err)
		}
		msg := decode(buf)
		receiver <- msg
	}
}

// RunUdpWriter sends messages from the outgoingMessages channel over UDP
func RunUdpWriter(outgoingMessages <-chan udpMessage, remote net.UDPAddr) {
	// Setup
	var conn, err = net.DialUDP("udp", nil, &remote)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	log.Default().Println("UDP writer running on address", remote)
	// Run the UDP writer
	for {
		var msg = <-outgoingMessages
		e := encode(msg)
		_, err = conn.Write(e)
		if err != nil {
			log.Println(err)
		}
	}
}

// init registers all types that are used in the udpMessage to gob
func init() {
	gob.Register(models.Hall{})
	gob.Register(models.Elevator{})
	gob.Register(models.ElevatorState{})
	gob.Register(models.Request{})
	gob.Register(models.Origin{})
}

// encode encodes a udpMessage to a byte slice
func encode(msg udpMessage) []byte {
	var buffer bytes.Buffer
	enc := gob.NewEncoder(&buffer)
	err := enc.Encode(msg)
	if err != nil {
		log.Fatal("Comms Failed to encode udpMessage:", err)
	}

	return buffer.Bytes()
}

// decode decodes a byte slice to a udpMessage
func decode(b []byte) udpMessage {
	var msg udpMessage
	dec := gob.NewDecoder(bytes.NewReader(b))
	err := dec.Decode(&msg)
	if err != nil {
		log.Fatal("Failed to decode udpMessage:", err)
	}
	return msg
}
