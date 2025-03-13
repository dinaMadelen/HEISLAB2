package distribution

import (
	"elevator/elevator_interface"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"syscall"
)

func DialBroadcastUDP(port int) net.PacketConn {
	s, err := syscall.Socket(syscall.AF_INET, syscall.SOCK_DGRAM, syscall.IPPROTO_UDP)
	if err != nil {
		fmt.Println("Error: Socket:", err)
	}
	syscall.SetsockoptInt(s, syscall.SOL_SOCKET, syscall.SO_REUSEADDR, 1)
	if err != nil {
		fmt.Println("Error: SetSockOpt REUSEADDR:", err)
	}
	syscall.SetsockoptInt(s, syscall.SOL_SOCKET, syscall.SO_BROADCAST, 1)
	if err != nil {
		fmt.Println("Error: SetSockOpt BROADCAST:", err)
	}
	syscall.Bind(s, &syscall.SockaddrInet4{Port: port})
	if err != nil {
		fmt.Println("Error: Bind:", err)
	}

	f := os.NewFile(uintptr(s), "")
	conn, err := net.FilePacketConn(f)
	if err != nil {
		fmt.Println("Error: FilePacketConn:", err)
	}
	f.Close()

	return conn
}

const bufSize = 1024

func Transmitter(port int, ch chan elevator_interface.Worldview) {
	conn := DialBroadcastUDP(port)
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", port))
	var currentWorldview elevator_interface.Worldview

	for {
		currentWorldview = <-ch
		jsonstr, _ := json.Marshal(currentWorldview)

		if len(jsonstr) > bufSize {
			panic(fmt.Sprintf(
				"Tried to send a message longer than the buffer size (length: %d, buffer size: %d)\n\t'%s'\n"+
					"Either send smaller packets, or go to network/bcast/bcast.go and increase the buffer size",
				len(jsonstr), bufSize, string(jsonstr)))
		}
		conn.WriteTo(jsonstr, addr)
	}
}

func Receiver(port int, ch chan elevator_interface.Worldview) {
	conn := DialBroadcastUDP(port)

	buf := make([]byte, bufSize)

	for {
		n, _, err := conn.ReadFrom(buf[0:])
		if err != nil {
			fmt.Println("Error reading from UDP:", err)
			return
		}
		var msg elevator_interface.Worldview
		json.Unmarshal(buf[:n], &msg)
		ch <- msg
	}
}
