package broadcastState

import (
	"Network-go/network/bcast"
	"fmt"
    "sanntids/cmd/localElevator/structs"
)

// BroadcastState transmits elevator state and hall orders in chunks
func BroadcastState(dataChan <-chan structs.ElevatorDataWithID, port int) {
	fmt.Printf("Starting data broadcaster on port %d\n", port)
	broadcastChan := make(chan structs.ElevatorDataWithID)
	
	go bcast.Transmitter(port, broadcastChan)
	
	for dataWithID := range dataChan {
		broadcastChan <- dataWithID
	}
}

// ReceiveState receives elevator state and hall orders
func ReceiveState(dataChan chan<- structs.ElevatorDataWithID, port int) {
	fmt.Printf("Starting data receiver on port %d\n", port)
	receiveChan := make(chan structs.ElevatorDataWithID)
	
	go bcast.Receiver(port, receiveChan)
	
	for receivedDataWithID := range receiveChan {
		dataChan <- receivedDataWithID
	}
}
