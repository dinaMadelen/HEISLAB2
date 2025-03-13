package network

import (
	"config"
	"log"
	"net"
	"strconv"
	"time"
	"color"
)

func Broadcast_server_address() {
	conn, err := net.DialUDP("udp", nil, &net.UDPAddr{
		IP:   net.IPv4(255, 255, 255, 255),
		Port: config.UDP_port,
	})
	if err != nil {
		log.Fatalf(color.Red + "Failed to create UDP connection: %v" + color.Reset, err)
	}
	defer conn.Close()

	for {
		_, err := conn.Write([]byte("Hello from server \n"))
		if err != nil {
			log.Printf(color.Red + "Failed to broadcast message: %v" + color.Reset, err)
			continue
		}
		time.Sleep(config.Broadcast_delay)
	}
}

func Find_server_address() string {
	addr, err := net.ResolveUDPAddr("udp", ":" + strconv.Itoa(config.UDP_port))
	if err != nil {
		log.Printf(color.Red + "Failed to resolve UDP address: %v" + color.Reset, err)
		return ""
	}

	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		log.Printf(color.Red + "Failed to listen for broadcast: %v" + color.Reset, err)
		return ""
	}

	defer conn.Close()
	conn.SetReadDeadline(time.Now().Add(config.Broadcast_delay * 4))

	buffer := make([]byte, 1024)
	n, addr, err := conn.ReadFromUDP(buffer)

	if err != nil {
		if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
			log.Printf(color.Orange + "Timeout occurred, no sender detected: %v" + color.Reset, err)
		}
		return ""
	}

	// If data was received
	if n > 0 {
		return addr.IP.String()
	}

	return ""
}
