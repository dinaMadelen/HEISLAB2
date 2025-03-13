package TCP

import (
	"fmt"
	"net"

	"Constants"
)

// Package TCP
// --------------------------------------------------------------------
//
// Implements TCP_Connection and TCP_Connection_Manager made by New_TCP_Connection(name, read_channel, connection)
// 															and New_TCP_Connection_Manager()
//
// TCP_Connection_Manager.Open_Server(PORT) opens a server on the PORT that automatically accepts
//		and adds any requested connection to the manager.
// TCP_Connection_Manager.Connect_Client(ADDRESS) attempts to connect a client to the address and
//		adds the resulting connection to the manager.
//
// TCP_Connection_Manager.Close_All() closes all connections. To be used when crashing gracefully.
//
// TCP_Connection_Manager.Add_Connection(TCP_Connection) will add the connection to the manager.
// TCP_Connection_Manager.Remove_Connection(TCP_Connection) will remove the connection from the manager.
//
// READING:
// From an individually created TCP_Connection: message <- TCP_Connection.Read_Channel
// A connection made by TCP_Connection_Manager: message <- TCP_Connection_Manager.Global_Read_Channel
//
// WRITING:
// Writing to an individual connection: TCP_Connection.Write_Channel <- message
// Broadcasting to all connections: use TCP_Connection_Manager.Broadcast(message)
// --------------------------------------------------------------------
// Connection

type TCP_Connection struct {
	// Public
	Write_Channel chan string
	Read_Channel  chan string

	// Protected
	close_channel   chan bool
	connection_name string
	connection      net.Conn
	split_handler   TCP_Split_Handler
}

// Creates a new TCP conneciton bound to a shared read channel.
func New_TCP_Connection(name string, read_channel chan string, connection net.Conn) TCP_Connection {
	write_channel := make(chan string, Constants.TCP_BUFFER_SIZE)
	close_channel := make(chan bool)

	return TCP_Connection{
		Write_Channel: write_channel,
		Read_Channel:  read_channel,
		close_channel: close_channel,

		connection_name: name,
		connection:      connection,

		split_handler: New_TCP_Split_Handler(), // New connection, no split read message yet.
	}
}

func (connection TCP_Connection) Close() {
	select {
	case <-connection.close_channel:
		// Connection was already closed!
		fmt.Printf("Error: Attempted to close closed channel %s!\n", connection.connection_name)
		return
	default:
		close(connection.close_channel)
	}
}

func (connection TCP_Connection) Get_Name() string {
	return connection.connection_name
}
