package communication

import (
	"G19_heis2/Heis/config"
	"G19_heis2/Heis/network/bcast"
	"time"
	"fmt"
)

const StatePort = 20011 //eksempel

func StartStateUpdate(elevator *config.Elevator, channels config.NetworkChannels){
	go bcast.Transmitter(StatePort, channels.StateTX)
	go bcast.Receiver(StatePort, channels.StateRX)

	go SendState(elevator, channels)
	go ListenState(channels)
}

func SendState(elev *config.Elevator, ch config.NetworkChannels){
	
	for {
		config.StateMutex.Lock()
		elev.Timestamp = time.Now().UnixNano()
		fmt.Printf("\n🔵 [SENDER] Elevator %s sending requests:\n", elev.ID)
		for f, row := range elev.Requests {
			fmt.Printf("   Floor %d: %v\n", f, row)
		}
		ch.StateTX <- elev
		config.StateMutex.Unlock()
		time.Sleep(1000*time.Millisecond)
	}
}

func ListenState(ch config.NetworkChannels){
	for receivedState := range ch.StateRX {
		config.StateMutex.Lock()
		// 🟢 Print kun requests som mottas
		fmt.Printf("\n🟢 [MOTTATT] State update from elevator %s:\n", receivedState.ID)
		for f, row := range receivedState.Requests {
			fmt.Printf("   Floor %d: %v\n", f, row)
		}

		if existingState, exists := config.GlobalState[receivedState.ID]; exists {
			
			if receivedState.Timestamp > existingState.Timestamp {
				config.GlobalState[receivedState.ID] = *receivedState
			}
		} else {
			config.GlobalState[receivedState.ID] = *receivedState
		}
		config.StateMutex.Unlock()
	}
}

