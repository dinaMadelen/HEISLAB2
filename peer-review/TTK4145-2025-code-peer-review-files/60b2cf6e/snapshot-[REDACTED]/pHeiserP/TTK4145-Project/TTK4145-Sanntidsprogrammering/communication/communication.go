package communication

import (
	"TTK4145-Heislab/Network-go/network/peers"
	"TTK4145-Heislab/single_elevator"
	"TTK4145-Heislab/worldview"
	"fmt"
)

func CommunicationHandler(

	elevatorID string,
	peerUpdateChannel <-chan peers.PeerUpdate,
	NewlocalElevatorChannel <-chan single_elevator.State,
	peerTXEnableChannel chan<- bool,
	IDPeersChannel chan<- []string,

) {

	localWorldView := worldview.InitializeWorldView(elevatorID)

	for {

		select {

		case newLocalElevator := <-NewlocalElevatorChannel:
			localWorldView.ElevatorStatusList[elevatorID] = newLocalElevator
			cabRequest := worldview.GetOurCAB(newLocalElevator)

		case peers := <-peerUpdateChannel:

			fmt.Printf("Peer update:\n")
			fmt.Printf("  Peers:    %q\n", peers.Peers)
			fmt.Printf("  New:      %q\n", peers.New)
			fmt.Printf("  Lost:     %q\n", peers.Lost)

			IDPeersChannel <- peers.Peers
		}
	}
}
