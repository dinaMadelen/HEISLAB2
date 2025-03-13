package network

import (
	"encoding/binary"
	"fmt"
	"log"
	"net"
)

type Packet struct {
	SequenceNumber uint32 //Contains information about sender and receiver like IP and port
	Timestamp      uint64 //Used to potentially tell something about network latency if we want
	Data           []byte //Contains the data
}

func SerializePacket(Message Packet) []byte { //Using the encoding library to encode messages to binary for effective sending using bits
	buffer := make([]byte, 16+len(Message.Data)) //Made a buffer of variable size, though size(Data) will likely be static
	binary.LittleEndian.PutUint32(buffer[0:4], Message.SequenceNumber)
	binary.LittleEndian.PutUint64(buffer[4:12], uint64(Message.Timestamp))
	copy(buffer[12:], Message.Data)

	return buffer
}

func DeSerializePacket(Data []byte) (Packet, error) { //Reads received []byte and de-codes into a Packet struct
	if len(Data) < 12 {
		return Packet{}, fmt.Errorf("Packet not of sufficient length\n")
	}

	packet := Packet{
		SequenceNumber: binary.LittleEndian.Uint32(Data[0:4]),
		Timestamp:      uint64(binary.LittleEndian.Uint64(Data[4:12])),
		Data:           Data[12:],
	}

	return packet, nil
}

func SendUDP(address *net.UDPAddr, messageToSend []byte) error {
	conn, err := net.DialUDP("udp", nil, address)
	if err != nil {
		fmt.Printf("Error dialing to UDP address\n")
		return err
	}

	defer conn.Close()

	_, err = conn.Write(messageToSend)

	return err
}

func ListenUDP(addr *net.UDPAddr, packetChan chan<- Packet) { //The post box is a pointer to another packet, which is supposed to always hold the newest message
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		log.Fatalf("Error listening: %v\n", err)
	}
	defer conn.Close()

	buffer := make([]byte, 1024)
	for {
		n, _, err := conn.ReadFromUDP(buffer)
		if err != nil {
			log.Printf("Error reading: %v\n", err)
			continue
		}

		newPacket, err := DeSerializePacket(buffer[:n])
		if err != nil {
			fmt.Printf("Could not de-serialize packet correctly\n")
			continue
		}

		packetChan <- newPacket
	}
}
