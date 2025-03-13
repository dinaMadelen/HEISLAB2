package network

import (
	"Constants"
	protocols "Network-Protocol/Network/Protocols"
	peer_to_peer "Network-Protocol/Peer_to_Peer"
)

type Node struct {
	p2p *peer_to_peer.P2P_Network

	name string // Elevator ID

	active_voter_names []string

	// When to Synchronize:
	// When you see someone who shouldn't be there
	// When you suspect someone is disconnected
	// Every so often (A few times a second?)
	active_synchronization     *protocols.Synchronization_Vote
	has_active_synchronization bool
}

func New_Node(name string) Node {

	network := Node{
		p2p: peer_to_peer.New_P2P_Network(),

		name: name,

		active_voter_names: make([]string, 0),

		active_synchronization:     &protocols.Synchronization_Vote{},
		has_active_synchronization: false,
	}

	go network.reader()

	return network
}

func (node Node) reader() {
	for{
		message := <-node.p2p.Read_Channel

		message_type := message.Message[0:4]

		switch message_type{
		case Constants.SYNC_MESSAGE:

		}
	}
}

func (_ Node) get_shared_state() string {
	return ""
}

func (network Node) get_voters() []string {
	result := make([]string, 1)

	result[0] = network.name

	return result
}
