package main

import (
	"fmt"
	"net"
)

func listenToServerUDP(address string) {
	// Listen to incoming packages
	addr, err := net.ResolveUDPAddr("udp", address)
	if err != nil {
		fmt.Println("Error:", err)
	}
	//socket
	conn, err := net.ListenUDP("udp", addr)
	fmt.Println("conn: ", conn)

	buffer := make([]byte, 1024)
	n, addr, err := conn.ReadFromUDP(buffer)
	if err != nil {
		fmt.Println("Error:", err)
	}
	fmt.Println("Message: ", string(buffer[0:n]))
	fmt.Println("IP: ", addr.IP)
	fmt.Println("Listening to")

}

func sendingToServerUDP(address string) {
	// try sending a message to the server IP on port 20011, listen to msg from server and print to terminal
	addr, err := net.ResolveUDPAddr("udp", address)

	if err != nil {
		fmt.Println("Error:", err)
	}

	conn, err := net.DialUDP("udp", nil, addr)

	if err != nil {
		fmt.Println("Error:", err)
	}

	for {
		_, err = conn.Write([]byte("Group [REDACTED]"))
		if err != nil {
			fmt.Println("Error:", err)
		}
	}

}
