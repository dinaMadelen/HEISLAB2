package main

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/elevio"
	"elevproj/Elevator/fsm"
	"elevproj/Elevator/masterChecks"
	"elevproj/Elevator/message"
	"elevproj/Elevator/timer"
	"elevproj/Master/master"
	"elevproj/Network/bcast"
	"elevproj/Network/localip"
	"elevproj/Network/peers"
	"elevproj/config"
	"fmt"
	"time"
)

func main() {
	var timerStart_chan = make(chan time.Duration)
	var master_chan = make(chan message.MasterMessage)
	var btn_chan = make(chan message.BtnPress)
	var stop_chan = make(chan bool)
	var peerUpdate_chan = make(chan peers.PeerUpdate)
	var peerTxEnable_chan = make(chan bool)
	var messageTx_chan = make(chan message.ReportMessage)
	var killMaster_chan = make(chan bool)
	//Vi trenger vel ikke tre av dem? Kan vi ikke bare ha en og sende stop tre ganger hvis vi skal stoppe heisen.
	//Lukker vi noen gang de andre kanalene? Skjer det automatisk når vi tar ctrl c
	var elevStopBcastTx_chan = make(chan bool)
	var elevStopBcastRx_chan = make(chan bool)
	var elevStopPeerRx_chan = make(chan bool)
	var elevTxenable_chan = make(chan bool)

	//Defining variables
	IP, _ := localip.LocalIP()
	activeElevator := elevator.MakeEmptyElevatorObject(IP)

	//Defining timers
	doorTimer := time.NewTimer(time.Duration(activeElevator.DoorOpenDuration))
	doorTimer.Stop()
	masterTimer := time.NewTimer(time.Duration(config.MasterTimeout))
	masterTimer.Stop()
	messageTimer := time.NewTimer(time.Duration(config.MessageTimeout))

	go timer.UpdateTimer(doorTimer, timerStart_chan, stop_chan)
	go bcast.Transmitter(int(config.ElevatorTxPort), elevStopBcastTx_chan, elevTxenable_chan, messageTx_chan, btn_chan)
	go bcast.Receiver(int(config.ElevatorRxPort), elevStopBcastRx_chan, master_chan)
	go peers.Transmitter(int(config.ElevatorPeerPort), IP, peerTxEnable_chan)
	go peers.Receiver(int(config.ElevatorPeerPort), elevStopPeerRx_chan, peerUpdate_chan)

	fmt.Println("starting init")
	//Initialisere i en loop med tre porter for å få tre heiser på en pc
	//Heller returnere initFloor og sette lik elevator for nå defineres et nytt objekt??
	drv_floors, drv_obstr, drv_stop, drv_buttons, newInitFloor := elevator.InitializeElevator("localhost:15657", activeElevator)
	activeElevator.LatestFloor = newInitFloor

	// Forslag til hva vi kan bytte ut masterCheck.CheckMaster ?
	var isMaster = masterchecks.CheckMaster(master_chan)
	if isMaster {
		go master.Master(killMaster_chan)
	}

	for {
		select {
		case <-messageTimer.C:
			message.SendMessageToMaster(activeElevator, messageTx_chan, elevTxenable_chan)
			messageTimer.Reset(time.Duration(config.MessageTimeout))

		case masterMessage := <-master_chan:
			masterTimer.Reset(time.Duration(config.MasterTimeout))

			var newBtn elevio.ButtonEvent
			activeElevator, newBtn = message.ProcessMasterMessage(masterMessage, activeElevator)

			if masterchecks.ShouldKillMaster(activeElevator) {
				killMaster_chan <- true
				time.Sleep(1500 * time.Millisecond)
			}

			if masterMessage.SetAllLights {
				elevator.SetAllLights(activeElevator)
			}
			//Updating requests based on if its hallrequest or cabrequest
			activeElevator = fsm.SetNewRequest(newBtn, activeElevator, timerStart_chan)

		case btnPressed := <-drv_buttons:
			// sending btnPress to master
			switch activeElevator.Case {
			case elevator.PeerElevator:
				fmt.Println("button pressed")
				elevTxenable_chan <- true
				btn_chan <- message.BtnPress{ID: activeElevator.ID, Btn: btnPressed}
			case elevator.SingleElevator:
				elevio.SetButtonLamp(btnPressed.Button, btnPressed.Floor, true)
				activeElevator.Requests[btnPressed.Floor][btnPressed.Button] = true
				fsm.OnRequestButtonPress(activeElevator, btnPressed.Floor, btnPressed.Button, timerStart_chan)
			}
		case currentPosition := <-drv_floors:
			elevio.SetFloorIndicator(currentPosition)
			if currentPosition != -1 {
				activeElevator = fsm.OnFloorArrival(activeElevator, currentPosition, timerStart_chan)
			}

		case obstructed := <-drv_obstr:
			fmt.Printf("obstruction: %+v\n", obstructed)
			//Denne er jeg usikker på om funker
			activeElevator.Behaviour = elevator.ObstructionActivated(activeElevator, obstructed, timerStart_chan)

		case stopped := <-drv_stop:
			var stopPressed bool
			activeElevator.Behaviour, stopPressed = elevator.StopActivated(activeElevator, stopped, timerStart_chan)
			if !stopPressed {
				activeElevator = fsm.OnDoorTimeout(activeElevator, timerStart_chan)
			}
			fmt.Printf("%+v\n", stopped)

		case <-doorTimer.C:
			activeElevator = fsm.OnDoorTimeout(activeElevator, timerStart_chan)

		case <-masterTimer.C:
			myRank := activeElevator.BackupInfo.RankMap[activeElevator.ID]
			//above er ikke det beste navnet
			higherRankConnections := elevator.FindHigherRankConnections(activeElevator, myRank)

			if len(activeElevator.Connections) <= 1 {
				//run singleElevator
				fmt.Println("-----------------\n\n\n SINGLE ELEVATOR \n\n\n -------------------------")
				activeElevator.Case = elevator.SingleElevator
				if activeElevator.BackupInfo.MasterID == activeElevator.ID {
					killMaster_chan <- true
				}

			} else if len(higherRankConnections) == myRank {
				fmt.Println("sending message to start master")
				fmt.Println("-----------------\n\n\n STARTING MASTER \n\n\n -------------------------")
				go master.Master(killMaster_chan)
			}

		case peerUpdate := <-peerUpdate_chan:
			fmt.Println("got a peer update")
			if peerUpdate.New != "" {
				activeElevator.Connections[peerUpdate.New] = true
				activeElevator.Case = elevator.PeerElevator
			}
			for _, IP := range peerUpdate.Lost {
				delete(activeElevator.Connections, IP)
			}
		}
	}
}
