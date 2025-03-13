package peer_to_peer

import (
	"Constants"
	"Network-Protocol/TCP"
	"Network-Protocol/UDP"
	"fmt"
	"time"
)

// Package peer_to_peer
//
// Handles Peer Detection over UDP and Communication over TCP
// Does not handle elevator detection (not all peers must be elevators, some may be listeners!)

type P2P_Network struct {
	Read_Channel chan P2P_Message

	TCP *TCP.TCP_Connection_Manager
	UDP UDP.UDP_Channel

	close_channel chan bool

	tcp_server_address string

	dependency_resolver *Dependency_Resolver
	dependency_handler  Dependency_Handler
	clock               Lamport_Clock
}

func New_P2P_Network() *P2P_Network {
	read_channel := make(chan P2P_Message, Constants.P2P_BUFFER_SIZE)

	tcp_manager := TCP.New_TCP_Connection_Manager()
	udp_channel := UDP.New_UDP_Channel()

	server_port := tcp_manager.Open_Server()
	server_address := UDP.Get_local_IP().String() + server_port
	fmt.Printf("Opened server at: %s\n", server_address)

	network := P2P_Network{
		Read_Channel: read_channel,

		TCP: tcp_manager,
		UDP: udp_channel,

		close_channel: make(chan bool),

		tcp_server_address: server_address,

		dependency_resolver: New_Dependency_Resolver(),
		dependency_handler:  New_Dependency_Handler(),
		clock:               New_Lamport_Clock(),
	}

	go network.peer_detection()
	go network.reader()

	return &network
}

func (network *P2P_Network) Close() {
	close(network.close_channel)
	network.TCP.Close_All()
	network.UDP.Close()
}

func (network *P2P_Network) Broadcast(message P2P_Message) {
	network.dependency_resolver.Emplace_New_Message(message)
	network.TCP.Broadcast(message.To_String())
}

func (network *P2P_Network) Send(message P2P_Message, recipient string) {
	network.TCP.Send(message.To_String(), recipient)
}

func (network *P2P_Network) Create_Message(message string, message_type P2P_Message_Type) P2P_Message {
	network.clock.Event()

	return New_P2P_Message(network.tcp_server_address, message_type, network.clock, message)
}

func (network *P2P_Network) reader() {
	for {
		select {
		case tcp_message := <-network.TCP.Global_Read_Channel:
			p2p_message := P2P_Message_From_String(tcp_message)

			network.clock.Update(p2p_message.Time)
			go network.publisher(p2p_message)

		case <-network.close_channel:
			return

		default:
			time.Sleep(time.Microsecond)
		}
	}
}

func (network *P2P_Network) publisher(message P2P_Message) {
	new_dependency := New_Dependency(message.Sender, message.Time)
	if network.dependency_handler.Has_Dependency(new_dependency) {
		// Already received this message, discard.
		return
	}
	network.dependency_handler.Add_Dependency(new_dependency)

	timeout := time.NewTimer(time.Second)

	// Wait for message dependency, then publish onto read channel.
	for {
		select {
		case <-timeout.C:
			return // Timed out.
		case <-network.close_channel:
			return // Network closed.
		default:
		}

		if network.dependency_handler.Has_Dependency(message.dependency) {
			if message.Type == MESSAGE {
				network.Read_Channel <- message
			} else {
				network.handle_special_case(message)
			}

			return
		}

		// Request the missing dependency.
		network.request_dependency(message.dependency)

		// Wait until more data is processed
		time.Sleep(20 * time.Millisecond)
	}
}

func (network *P2P_Network) request_dependency(dependency Dependency) {
	message := network.Create_Message(dependency.To_String(), REQUEST_MISSING_DEPENDENCY)
	network.Send(message, dependency.Dependency_Owner)
}

func (network *P2P_Network) peer_detection() {
	renew_presence_ticker := time.NewTicker(Constants.UDP_WAIT_BEFORE_TRANSMITTING_AGAIN)

	for {
		select {
		case <-network.close_channel:
			return // P2P Connection closed

		case <-renew_presence_ticker.C:
			network.announce_presence()

		case address := <-network.UDP.Read_Channel:
			if !network.TCP.Does_Connection_Exist(address) {
				network.TCP.Connect_Client(address)
			}
		}
	}
}

func (network *P2P_Network) announce_presence() {
	network.UDP.Broadcast(network.tcp_server_address)
}
