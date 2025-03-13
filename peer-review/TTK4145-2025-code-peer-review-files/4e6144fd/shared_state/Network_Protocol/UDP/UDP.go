package UDP

import (
	"fmt"
	"net"
	"time"

	"golang.org/x/net/ipv4"

	"Constants"
)

// Package UDP
//-----------------------------------------------------------------
//
// Implements a struct UDP_Channel, connected to Multicast (239.255.255.255), made by New_UDP_Channel().
//
// WRITING:
// By default on
// Use Channel.Write_Channel <- message, to write onto the channel
//
// READING:
// Use message <- Channel.Read_Channel, to read from the channel after Start_Reading() has been called
//
// Note: There may be messages read in Read_Channel after Stop_Reading has been called.
//
// ----------------------------------------------------------------

type UDP_Channel struct {
	// Public
	Write_Channel chan string
	Read_Channel  chan string

	// Protected
	quit_channel chan bool
}

func (channel UDP_Channel) Close() {
	close(channel.quit_channel)
}

func (channel UDP_Channel) Broadcast(message string) {
	channel.Write_Channel <- message
}

func New_UDP_Channel() UDP_Channel {
	channel_write := make(chan string, 1024)
	channel_read := make(chan string, 1024)
	channel_quit := make(chan bool)

	channel := UDP_Channel{
		Write_Channel: channel_write,
		Read_Channel:  channel_read,
		quit_channel:  channel_quit,
	}

	channel.create_UDP_client()
	err := channel.create_UDP_server()
	if err != nil {
		fmt.Printf("Error when starting UDP server: %s\n", err.Error())
	}

	return channel
}

// --------------------------------------------------------------------

func Get_local_IP() net.IP {
	connection, err := net.Dial("udp", "255.255.255.255:1")
	if err != nil {
		panic(err)
	}
	defer connection.Close()

	localaddr := connection.LocalAddr().(*net.UDPAddr)

	return localaddr.IP
}

func (Channel UDP_Channel) udp_client(connection *net.UDPConn) {
	defer connection.Close()

	// Handle incoming write requests
	for {
		select {
		case message := <-Channel.Write_Channel:
			data := []byte(message)
			connection.Write(data)
		case <-Channel.quit_channel:
			return
		}

		time.Sleep(time.Microsecond)
	}
}

func (Channel UDP_Channel) udp_server(connection *ipv4.PacketConn) {
	defer connection.Close()

	ticker_read_UDP := time.NewTicker(Constants.UDP_WAIT_BEFORE_READING_AGAIN)

	for {
		select {
		case <-ticker_read_UDP.C:
			deadline := time.Now().Add(Constants.UDP_READ_DEADLINE)
			connection.SetReadDeadline(deadline)

			data := make([]byte, 1024)
			bytes_received, _, _, err := connection.ReadFrom(data)

			if err == nil {
				message := string(data[0:bytes_received])
				Channel.Read_Channel <- message
			}
		case <-Channel.quit_channel:
			return
		}

		time.Sleep(time.Microsecond) // Needed to make time advance in the VM so that the channels can close.
	}
}

func (channel *UDP_Channel) create_UDP_client() {
	addr, err := net.ResolveUDPAddr("udp", Constants.UDP_BROADCAST_IP_PORT)
	if err != nil {
		panic(err)
	}

	connection, err := net.DialUDP("udp", nil, addr)
	if err != nil {
		panic(err)
	}

	go channel.udp_client(connection)
}

// func create_UDP_server, to be called by a function that creates an UDP_Channel
func (channel *UDP_Channel) create_UDP_server() error {
	addr, err := net.ResolveUDPAddr("udp", Constants.UDP_BROADCAST_IP_PORT)
	if err != nil {
		panic(err)
	}

	connection, err := net.ListenPacket("udp4", Constants.UDP_BROADCAST_IP_PORT)
	if err != nil {
		return err
	}

	packet_conn := ipv4.NewPacketConn(connection)

	nifi, err := net.Interfaces()
	if err != nil {
		return err
	}

	err = packet_conn.SetMulticastLoopback(true)
	if err != nil {
		return err
	}

	for _, ifi := range nifi {
		packet_conn.JoinGroup(&ifi, addr)
	}

	go channel.udp_server(packet_conn)

	return nil
}
