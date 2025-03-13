package communication

import (
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/network/bcast"
	"G19_heis2/Heis/network/peers"
	"fmt"
	"time"
)

type HeartBeat struct {
	ElevatorID string
	Timestamp  time.Time
}

func StartHeartBeat(Elevator *config.Elevator, tx chan HeartBeat, rx chan HeartBeat) {
	peerLastSeen := make(map[string]time.Time)

	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	go peers.Transmitter(15657, Elevator.ID, peerTxEnable) // Samme port som i network-koden
	go peers.Receiver(15657, peerUpdateCh)

	go bcast.Transmitter(30000, tx)
	go bcast.Receiver(30000, rx)

	go SendHeartBeat(Elevator, tx)

	go ListenHeartBeat(rx, peerLastSeen, peerUpdateCh)

	fmt.Println("Heartbeat system started for", Elevator.ID)
}

func SendHeartBeat(elevator *config.Elevator, tx chan HeartBeat) {
	for {
		tx <- HeartBeat{
			ElevatorID: elevator.ID,
			Timestamp:  time.Now(),
		}
		time.Sleep(500 * time.Millisecond)
	}
}

func ListenHeartBeat(rx chan HeartBeat, peerLastSeen map[string]time.Time, peerUpdateCh chan peers.PeerUpdate) {
	for {
		select {
		case hb := <-rx:
			peerLastSeen[hb.ElevatorID] = time.Now()
			//fmt.Printf("Heartbeat recieved from %s\n", hb.ElevatorID)

		case peerUpdate := <-peerUpdateCh:
			fmt.Println("Peer update:")
			fmt.Printf("  Peers:    %q\n", peerUpdate.Peers)
			fmt.Printf("  New:      %q\n", peerUpdate.New)
			fmt.Printf("  Lost:     %q\n", peerUpdate.Lost)

			// Hvis en peer blir borte, fjern den fra lastSeen-mappen
			for _, lostPeer := range peerUpdate.Lost {
				fmt.Printf("Heis %s har gått offline!\n", lostPeer)
				delete(peerLastSeen, lostPeer)
			}

		default: // Sjekker for offline heiser hvert 5ms
			time.Sleep(5 * time.Millisecond)
			for id, lastSeen := range peerLastSeen {
				if time.Since(lastSeen) > 3*time.Second { // Timeout på 1 sek
					fmt.Printf("Heis %s har gått offline!\n", id)
					delete(peerLastSeen, id) //
				}
			}
		}
	}
}
