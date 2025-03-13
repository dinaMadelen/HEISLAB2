package TCP

import (
	"Constants"
	"fmt"
	"sync"
)

// TCP Connection Manager
// Implements creating TCP servers and clients and storing them onto a connection.
// Automatic disconnection is handled by TCP Handler
type TCP_Connection_Manager struct {
	mu          sync.Mutex
	Connections map[string]TCP_Connection // TCP Connection defined in TCP_internal.go

	Global_Read_Channel chan string
	stop_server_channel chan bool
}

func New_TCP_Connection_Manager() *TCP_Connection_Manager {
	return &TCP_Connection_Manager{
		Connections:         make(map[string]TCP_Connection),
		Global_Read_Channel: make(chan string, Constants.TCP_BUFFER_SIZE),
		stop_server_channel: make(chan bool),
	}
}

func (manager *TCP_Connection_Manager) Add_Connection(connection TCP_Connection) {
	manager.mu.Lock()
	defer manager.mu.Unlock()

	if manager.Does_Connection_Exist(connection.connection_name) {
		connection.Close()
		return
	}

	manager.Connections[connection.connection_name] = connection
	fmt.Printf("Connection %s added!\n", connection.connection_name)
}

func (manager *TCP_Connection_Manager) Remove_Connection(connection TCP_Connection) {
	manager.mu.Lock()
	defer manager.mu.Unlock()

	delete(manager.Connections, connection.connection_name)
	fmt.Printf("Connection %s removed!\n", connection.connection_name)
}

func (manager *TCP_Connection_Manager) Does_Connection_Exist(connection_name string) bool {
	_, ok := manager.Connections[connection_name]
	return ok
}

func (manager *TCP_Connection_Manager) Close_All() {
	manager.mu.Lock()
	defer manager.mu.Unlock()

	close(manager.stop_server_channel)

	for _, connection := range manager.Connections {
		connection.Close()
	}
}

func (manager *TCP_Connection_Manager) Open_Server() string {
	address_channel := make(chan string)

	go manager.create_TCP_Server(address_channel)

	return <-address_channel
}

func (manager *TCP_Connection_Manager) Connect_Client(address string) {
	go manager.create_TCP_Client(address)
}

func (manager *TCP_Connection_Manager) Broadcast(message string) {
	manager.mu.Lock()
	defer manager.mu.Unlock()

	for _, connection := range manager.Connections {
		connection.Write_Channel <- message
	}
}

func (manager *TCP_Connection_Manager) Send(message string, recipient string) {
	if !manager.Does_Connection_Exist(recipient) {
		fmt.Printf("Error, connection %s did not exist! Failed sending %s\n", recipient, message)
		manager.Connect_Client(recipient)
		return
	}

	manager.Connections[recipient].Write_Channel <- message
}
