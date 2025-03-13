package TCP

import (
	"fmt"
	"net"
)

func (connection_manager *TCP_Connection_Manager) setup_TCP_Connection(connection net.Conn) {
	// We don't want to delay, send everything asap.
	tcp_conn := connection.(*net.TCPConn)
	tcp_conn.SetNoDelay(true)

	// Add the incoming connection to the connection manager
	connection_name := connection.RemoteAddr().String()
	connection_object := New_TCP_Connection(connection_name, connection_manager.Global_Read_Channel, connection)
	connection_manager.Add_Connection(connection_object)

	// Hand over the accepted connection to a TCP Connection Handler.
	go connection_object.handle_TCP_Connection(connection_manager)
}

func (connection_manager *TCP_Connection_Manager) tcp_listener(listener *net.Listener) {
	// Accept all incoming connections
	for {
		connection, err := (*listener).Accept()
		if err != nil {
			fmt.Println("Error in accepting connection: ", err)
			return
		}

		connection_manager.setup_TCP_Connection(connection)
	}
}

func (connection_manager *TCP_Connection_Manager) create_TCP_Server(addr_channel chan string) {
	// Create a listener
	listener, err := net.Listen("tcp", ":0")
	if err != nil {
		panic(err)
	}
	defer listener.Close()

	// Address from a listener is on the form [::]:xxxxx
	const port_offset = 4
	addr_channel <- listener.Addr().String()[port_offset:]

	go connection_manager.tcp_listener(&listener)

	<-connection_manager.stop_server_channel
}

func (connection_manager *TCP_Connection_Manager) create_TCP_Client(address string) {
	connection, err := net.Dial("tcp", address)
	if err != nil {
		fmt.Println("Error setting up connection: ", err)
		return
	}

	connection_manager.setup_TCP_Connection(connection)
}
