package primary

import (
	"fmt"
	. "source/config"
	"source/localElevator/elevio"
	"source/primary/assigner"
	"time"
)

func Run(
	peerUpdateChan <-chan PeerUpdate,
	elevStateChan <-chan Elevator,
	becomePrimaryChan <-chan Worldview,
	worldviewTXChan chan<- Worldview,
	worldviewRXChan <-chan Worldview,
	requestFromElevChan <-chan Order,
	orderToElevChan chan<- Order,
	hallLightsChan chan<- [][]bool,
	myId string) {

	MapActionChan := make(chan FleetAccess, 10)
	ReadMapChan := make(chan map[string]Elevator, 2)

	var worldview Worldview
	worldview.FleetSnapshot = make(map[string]Elevator)
  	obstructedElevators := make([]string, NUM_ELEVATORS)
	//Init hallLights matrix
	hallLights := make([][]bool, NUM_FLOORS)
	for i := range hallLights {
		hallLights[i] = make([]bool, NUM_BUTTONS-1)
	}

	//Owns and handles access to fleet map
	go fleetAccessManager(MapActionChan)

	select {
	case wv := <-becomePrimaryChan:
		fmt.Println("Taking over as Primary")
		worldview = wv
		//drain(elevStateChan) //FIX FLUSHING OF CHANNELS(?)
		MapActionChan <- FleetAccess{Cmd: "write all", ElevMap: worldview.FleetSnapshot}
		obstructionTimers := make(map[string]*time.Timer)
		HeartbeatTimer := time.NewTicker(T_HEARTBEAT)
		defer HeartbeatTimer.Stop()
    	
		//primaryLoop:
		for {
			select {
			case worldview.PeerInfo = <-peerUpdateChan:
				//If elev lost: Reassign lost orders
				printPeers(worldview.PeerInfo)
				lost := worldview.PeerInfo.Lost
				if len(lost) != 0 {
					ReassignHallOrders(worldview, MapActionChan, orderToElevChan, Reassignment{Cause: Disconnected})
				}

			case elevUpdate := <-elevStateChan:
				//Request write
				MapActionChan <- FleetAccess{Cmd: "write one", Id: elevUpdate.Id, Elev: elevUpdate}
				//has a race condition but works fine
				updateHallLights(worldview, hallLights, MapActionChan, hallLightsChan)
			
        		// ------ OBSTRUCTION ------- //
				//If elevator is obstructed for 3 sec, reassign hall orders
				if elevUpdate.Obstructed {
					obstructedElevators = append(obstructedElevators, elevUpdate.Id)
					//If no timer, start one
					_, timerExists := obstructionTimers[elevUpdate.Id]
					if !timerExists{
						timer := time.AfterFunc(T_OBSTRUCTED_PRIMARY, func() {
							reassignmentDetails := Reassignment{Cause: Obstructed, ObsId: obstructedElevators[len(obstructedElevators)-1]}
							ReassignHallOrders(worldview, MapActionChan, orderToElevChan, reassignmentDetails)
						})
						obstructionTimers[elevUpdate.Id] = timer
					}
				} else {
					//if ID in obstructedElevatorIds, pop id and stop timer
					// If the elevator is no longer obstructed, check if its ID is in the list of obstructed elevators
					for i, id := range obstructedElevators {
						if id == elevUpdate.Id {
							// If found, remove it from the slice
							obstructedElevators = append(obstructedElevators[:i], obstructedElevators[i+1:]...)
							//obstructedElevators = slices.Delete(obstructedElevators,i,i+1)
							// Stop the timer if it's active
							if timer, exists := obstructionTimers[elevUpdate.Id]; exists {
								timer.Stop()
								delete(obstructionTimers, elevUpdate.Id)
							}
							break
						}
					}
				}
  
			case request := <-requestFromElevChan:
				MapActionChan <- FleetAccess{Cmd: "read", ReadCh: ReadMapChan}
				select {
					case worldview.FleetSnapshot = <-ReadMapChan:
				}
				AssignedId := assigner.ChooseElevator(worldview.FleetSnapshot, worldview.PeerInfo.Peers, request)
				orderToElevChan <- OrderConstructor(AssignedId, request.Floor, request.Button)
				fmt.Printf("Assigned elevator %s to order\n", AssignedId)

			case <-HeartbeatTimer.C:
				MapActionChan <- FleetAccess{Cmd: "read", ReadCh: ReadMapChan}
				select {
				case worldview.FleetSnapshot = <-ReadMapChan:
				}
				worldviewTXChan <- worldview

			/* case receivedWV := <-worldviewRXChan:
				receivedId := receivedWV.PrimaryId
				fmt.Print(receivedId)
				if receivedId < myId {
					fmt.Printf("Primary: %s, taking over\n", receivedId)
					break primaryLoop */
			 //defere break om mulig?
			}
		}
	}
}


func fleetAccessManager(mapActionChan <-chan FleetAccess) {
	fleet := make(map[string]Elevator) // Real fleet map. All others are snapshots of this
	for {
		select {
		case newAction := <-mapActionChan:
			switch newAction.Cmd {
			case "read":
				deepCopy := make(map[string]Elevator, len(fleet))
				for key, value := range fleet {
					deepCopy[key] = value
				}
				newAction.ReadCh <- deepCopy
			case "write one":
				fleet[newAction.Id] = newAction.Elev
			case "write all":
				fleet = newAction.ElevMap
			}
		}
	}
}

/* MAYBE implement function that owns hallLight state to avoid "trivial" race condition. Would be similar to fleetAccessManager
NOT 1st priority.  */

func updateHallLights(wv Worldview, hallLights [][]bool, MapActionChan chan<- FleetAccess, hallLightsChan chan<- [][]bool) {
	readChan := make(chan map[string]Elevator, 1)
	defer close(readChan)
	//Request read
	MapActionChan <- FleetAccess{Cmd: "read", ReadCh: readChan}
	shouldUpdate := false
	prevHallLights := make([][]bool, NUM_FLOORS)
	for floor := range hallLights {
		prevHallLights[floor] = make([]bool, NUM_BUTTONS-1)
		copy(prevHallLights[floor], hallLights[floor]) // Copy row data
		for btn := range NUM_BUTTONS - 1 {
			hallLights[floor][btn] = false
		}
	}
	select {
	case fleetSnapshot := <-readChan:
		wv = WorldviewConstructor(wv.PrimaryId, wv.PeerInfo, fleetSnapshot)
		for _, id := range wv.PeerInfo.Peers {
			orderMatrix := wv.FleetSnapshot[id].Orders
			for floor, floorOrders := range orderMatrix {
				for btn, isOrder := range floorOrders {
					if isOrder && btn != int(elevio.BT_Cab) {
						hallLights[floor][btn] = true
					}
				}
			}
		}
	}	
  for floor := range NUM_FLOORS {
		for btn := range NUM_BUTTONS - 1 {
			if prevHallLights[floor][btn] != hallLights[floor][btn] {
				shouldUpdate = true

			}
		}
	}
	if shouldUpdate {
		hallLightsChan <- hallLights
	}
}

func ReassignHallOrders(wv Worldview, MapAccessChan chan FleetAccess, orderToElevChan chan<- Order, reassign Reassignment){
	readChan := make(chan map[string]Elevator, 1)
	defer close(readChan)
	MapAccessChan <- FleetAccess{Cmd: "read", ReadCh: readChan}

	select {
	case fleetSnapshot := <-readChan:
		wv := WorldviewConstructor(wv.PrimaryId, wv.PeerInfo, fleetSnapshot)

		switch reassign.Cause{
		case Disconnected:
			for _, lostId := range wv.PeerInfo.Lost {
			orderMatrix := wv.FleetSnapshot[lostId].Orders
			for floor, floorOrders := range orderMatrix {
				for btn, isOrder := range floorOrders {
					if isOrder && btn != int(elevio.BT_Cab) {
						lostOrder := Order{
							Id:     lostId,
							Floor:  floor,
							Button: btn,
						}
						lostOrder.Id = assigner.ChooseElevator(wv.FleetSnapshot, wv.PeerInfo.Peers, lostOrder)
						orderToElevChan <- lostOrder
					}
				}
			}
		}
		case Obstructed:
			orderMatrix := wv.FleetSnapshot[reassign.ObsId].Orders
			for floor, floorOrders := range(orderMatrix){
			  for btn, isOrder := range(floorOrders){
				if isOrder && btn != int(elevio.BT_Cab){
				  lostOrder:=Order{
						Id: reassign.ObsId,
						Floor: floor,
						Button: btn,
					  }
				  lostOrder.Id = assigner.ChooseElevator(wv.FleetSnapshot, wv.PeerInfo.Peers, lostOrder)
				  orderToElevChan <- lostOrder
				}
			  }
			}
		}
	}
}

func printPeers(p PeerUpdate) {
	fmt.Printf("Peer update:\n")
	fmt.Printf("  Peers:    %q\n", p.Peers)
	fmt.Printf("  New:      %q\n", p.New)
	fmt.Printf("  Lost:     %q\n", p.Lost)
}
