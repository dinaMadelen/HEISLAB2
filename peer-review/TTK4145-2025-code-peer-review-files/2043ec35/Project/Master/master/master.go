package master

import (
	"elevproj/Elevator/message"
	mastermessageprocessing "elevproj/Master/masterMessageProcessing"
	masterobject "elevproj/Master/masterObject"
	"elevproj/Master/requestDistribution"
	"elevproj/Network/bcast"
	"elevproj/Network/peers"
	"elevproj/config"
	"fmt"
	"time"
)

func Master(killMaster chan bool) {
	//Defining MasterChannels
	var elevatorReport_chan = make(chan message.ReportMessage)
	var sendUpdate_chan = make(chan message.MasterMessage)
	var btn_chan = make(chan message.BtnPress)
	var masterPeerUpdate_chan = make(chan peers.PeerUpdate)
	var stopBcastTx_chan = make(chan bool)
	var stopBcastRx_chan = make(chan bool)
	var stopPeerRx_chan = make(chan bool)
	var masterTxenable_chan = make(chan bool)

	//The master goroutines
	go bcast.Transmitter(int(config.MasterTxPort), stopBcastTx_chan, masterTxenable_chan, sendUpdate_chan)
	go bcast.Receiver(int(config.MasterRxPort), stopBcastRx_chan, elevatorReport_chan, btn_chan)
	go peers.Receiver(int(config.MasterRxPeerPort), stopPeerRx_chan, masterPeerUpdate_chan)

	activeMaster := masterobject.MakeEmptyMasterObject()
	// packet loss: vil denne linja vente til den får en melding
	initialReport := <-elevatorReport_chan
	activeMaster = mastermessageprocessing.GetBackupInfo(activeMaster, initialReport)

	var masterMessage message.MasterMessage
	sendMessageTimer := time.NewTimer(time.Duration(config.MasterMessageTimeout))

	for {
		select {
		case newReport := <-elevatorReport_chan:
			var HRAready bool
			HRAready, activeMaster = mastermessageprocessing.ProcessMessage(newReport, activeMaster)

			masterMessage.SetAllLights = HRAready
			if HRAready {
				activeMaster.DistributedHallRequests = requestdistribution.RunHRA(activeMaster.FullHallRequests, activeMaster.ElevatorReports)
			}

		case btnPressed := <-btn_chan:
			fmt.Println("Master got button")
			activeMaster = mastermessageprocessing.ProcessButton(btnPressed, activeMaster)

		case <-sendMessageTimer.C:
			fmt.Println("my rank: ", activeMaster.RankMap[activeMaster.ID])
			fmt.Println("masters distributed: ", activeMaster.DistributedHallRequests)
			masterMessage = mastermessageprocessing.UpdateMessage(masterMessage, activeMaster)
			fmt.Println(masterMessage.DistributedHallRequests)
			masterTxenable_chan <- true
			sendUpdate_chan <- masterMessage
			sendMessageTimer.Reset(config.MasterMessageTimeout)

		case peerUpdate := <-masterPeerUpdate_chan:
			//Det her burde bli en funksjon, men usikker hvordan
			fmt.Println("got a peer update")
			_, exists := activeMaster.RankMap[peerUpdate.New]
			if peerUpdate.New != "" {
				activeMaster.Connections[peerUpdate.New] = true
				if !exists {
					newRank := len(activeMaster.RankMap)
					activeMaster.RankMap[peerUpdate.New] = newRank
					fmt.Println("updated rankmap: ", activeMaster.RankMap)
				}
			}
			for _, IP := range peerUpdate.Lost {
				delete(activeMaster.Connections, IP)
			}

		case <-killMaster:
			fmt.Println("-----------------\n\n\n KILLING MASTER \n\n\n -------------------------")
			//burde stoppe alle goroutines
			close(stopBcastTx_chan)
			fmt.Println("first killed")
			close(stopBcastRx_chan)
			fmt.Println("second killed")
			close(stopPeerRx_chan)
			fmt.Println("returning masterMain")
			return

		}
	}

}
