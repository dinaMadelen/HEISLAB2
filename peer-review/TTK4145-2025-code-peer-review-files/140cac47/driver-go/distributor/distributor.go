package distributor

import (
	"Driver-go/assigner"
	"Driver-go/config"
	"Driver-go/elevator"
	"Driver-go/elevio"
	"Driver-go/network/peers"
	"time"
	"strconv"
)

// Initialize elevator distributor
func elevatorDistributorInit(id string) config.ElevatorDistributor{
	requests := make([][]config.RequestState, 4)
	for floor := range requests{
		requests[floor] = make([]config.RequestState, 3)
	}
	return config.ElevatorDistributor{Requests: requests, ID: id, Floor:0, Behaviour: config.Idle}

}
// Broadcast the current state of all elevators to a specified channel
func broadcast(elevators []*config.ElevatorDistributor, ch_transmit chan <- []config.ElevatorDistributor){
	temporaryElevators := make([]config.ElevatorDistributor, 0)
	for _, elevator := range elevators{
		temporaryElevators = append(temporaryElevators, *elevator)
	}
	ch_transmit <- temporaryElevators
	time.Sleep(25*time.Millisecond)
}

// Reinitializes an elevator with a given ID
func reinitializeElevator(elevators []*config.ElevatorDistributor, id int) {
    for _, elev := range elevators {
        if elev.ID == strconv.Itoa(id) {
            *elev = elevatorDistributorInit(strconv.Itoa(id))
            break
        }
    }
}

// Distribuing orders among the elevators
func Distributor(
	id int,
	ch_newLocalOrder chan elevio.ButtonEvent,
	ch_newLocalState chan elevator.Elevator, 
	ch_msgFromNetwork chan []config.ElevatorDistributor, 
	ch_msgToNetwork chan []config.ElevatorDistributor, 
	ch_orderToLocal chan elevio.ButtonEvent, 
	ch_peerUpdate chan peers.PeerUpdate, 
	ch_watchdogStuckReset chan bool , 
	ch_watchdogStuckSignal chan bool, 
	ch_clearLocalHallOrders chan bool){


	elevators := make([]*config.ElevatorDistributor, 0)
	thisElevator := new(config.ElevatorDistributor)
	*thisElevator = elevatorDistributorInit(strconv.Itoa(id))
	elevators = append(elevators, thisElevator)

	connectTimer := time.NewTimer(time.Duration(config.ReconnectTimer)*time.Second)

	// Check the network for new elevators, handles receiving the new elevators states
	select{
	case newElevators := <- ch_msgFromNetwork:
		for _, elev := range newElevators{
			if elev.ID == elevators[config.LocalElevator].ID{
				for floor := range elev.Requests{
					if elev.Requests[floor][config.Cab] == config.Confirmed || elev.Requests[floor][config.Cab] == config.Order{
						ch_newLocalOrder <- elevio.ButtonEvent{Floor: floor, Button: elevio.ButtonType(int(config.Cab))}
					}
				}
			}
		}
		break 

	case <- connectTimer.C:
		break
	}

	// Distributes orders among the elevators on the network
	for{
		select{
		case newOrder := <- ch_newLocalOrder:
			assigner.AssignOrder(elevators, newOrder)
			if elevators[config.LocalElevator].Requests[newOrder.Floor][newOrder.Button] == config.Order{
				broadcast(elevators, ch_msgToNetwork)
				elevators[config.LocalElevator].Requests[newOrder.Floor][newOrder.Button] = config.Confirmed
				setAllLights(elevators,id)
				ch_orderToLocal <- newOrder
			}
			broadcast(elevators, ch_msgToNetwork)
			setAllLights(elevators,id)

		case newState := <- ch_newLocalState:
			if newState.Floor != elevators[config.LocalElevator].Floor || newState.Behave == elevator.Idle || newState.Behave == elevator.DoorOpen{
				elevators[config.LocalElevator].Behaviour = config.Behaviour(int(newState.Behave))
				elevators[config.LocalElevator].Floor = newState.Floor
				elevators[config.LocalElevator].Direction = config.Direction(int(newState.Direction))
				ch_watchdogStuckReset <- false
			}
			for floor := range elevators[config.LocalElevator].Requests{
				for button := range elevators[config.LocalElevator].Requests[floor]{
					if !newState.Requests[floor][button] && elevators[config.LocalElevator].Requests[floor][button] == config.Confirmed{
						elevators[config.LocalElevator].Requests[floor][button] = config.Complete
					}
					if elevators[config.LocalElevator].Behaviour != config.Unavailable && newState.Requests[floor][button] && elevators[config.LocalElevator].Requests[floor][button] != config.Confirmed{
						elevators[config.LocalElevator].Requests[floor][button] = config.Confirmed
					}
				}
				
			}
			setAllLights(elevators,id)
			broadcast(elevators, ch_msgToNetwork)
			removeCompletedOrders(elevators)
			
		case newElevators := <- ch_msgFromNetwork:
			if len(newElevators) > 0 {
				updateElevators(elevators,newElevators)
			}
			assigner.ReassignOrders(elevators, ch_newLocalOrder)
			for _, newElev := range newElevators{
				elevExists := false
				for _, elev := range elevators{
					if elev.ID == newElev.ID{
						elevExists = true
						break
					}
				}
				if !elevExists{
					addNewElevator(&elevators,newElev)
				}
			}
			extractNewOrder := confirmedNewOrder(elevators[config.LocalElevator])
			setAllLights(elevators,id)
			removeCompletedOrders(elevators)
			if extractNewOrder != nil{
				tempOrder := elevio.ButtonEvent{
					Button : elevio.ButtonType(extractNewOrder.Button),
					Floor : extractNewOrder.Floor}
				ch_orderToLocal <- tempOrder
				broadcast(elevators, ch_msgToNetwork)
			}

		case peer := <- ch_peerUpdate:
			if len(peer.Lost) != 0{
				for _, stringLostId := range peer.Lost{
					for _,elev := range elevators {
						if stringLostId == elev.ID{
							elev.Behaviour = config.Unavailable
						}
						assigner.ReassignOrders(elevators, ch_newLocalOrder)
						for floor := range elev.Requests{
							for button := 0; button < len(elev.Requests[floor])-1 ; button++{
								elev.Requests[floor][button] = config.None
							}
						}
					}
				}
			}
			setAllLights(elevators,id)
			broadcast(elevators, ch_msgToNetwork)

		case <- ch_watchdogStuckSignal: // Detection of stuck elevator
			elevators[config.LocalElevator].Behaviour = config.Unavailable
			broadcast(elevators, ch_msgToNetwork)
			for floor := range elevators[config.LocalElevator].Requests{
				for button := 0; button < len(elevators[config.LocalElevator].Requests[floor])-1;button++{
					elevators[config.LocalElevator].Requests[floor][button] = config.None
				}
			}
			setAllLights(elevators,id)
			ch_clearLocalHallOrders <- true
			reinitializeElevator(elevators, id)
            broadcast(elevators, ch_msgToNetwork)
		}
	}
}

// Remove completed orders from the elevator
func removeCompletedOrders(elevators []*config.ElevatorDistributor){
	for _, elev := range elevators{
		for floor := range elev.Requests{
			for button := range elev.Requests[floor]{
				if elev.Requests[floor][button] == config.Complete{
					elev.Requests[floor][button] = config.None
				}
			}
		}
	}
}

// Update state of local elevator based on new elevator states 
func updateElevators(elevators []*config.ElevatorDistributor, newElevators []config.ElevatorDistributor){
	if elevators[config.LocalElevator].ID != newElevators[config.LocalElevator].ID{
		for _,elev := range elevators{
			if elev.ID == newElevators[config.LocalElevator].ID{
				for floor := range elev.Requests{
					for button := range elev.Requests[floor]{
						if !(elev.Requests[floor][button] == config.Confirmed && newElevators[config.LocalElevator].Requests[floor][button] == config.Order){
							elev.Requests[floor][button] = newElevators[config.LocalElevator].Requests[floor][button]
						}
						elev.Floor = newElevators[config.LocalElevator].Floor
						elev.Direction = newElevators[config.LocalElevator].Direction
						elev.Behaviour = newElevators[config.LocalElevator].Behaviour
					}
				}
			}
		}
		for _, newElev := range newElevators{
			if newElev.ID == elevators[config.LocalElevator].ID{
				for floor := range newElev.Requests{
					for button := range newElev.Requests[floor]{
						if elevators[config.LocalElevator].Behaviour != config.Unavailable{
							if newElev.Requests[floor][button] == config.Order {
								(*elevators[config.LocalElevator]).Requests[floor][button] = config.Order
							}
						}
					}
				}
			}
		}
	}
}

// Add a new elevator to the network
func addNewElevator (elevators *[]* config.ElevatorDistributor, newElevator config.ElevatorDistributor) {
	tempElev := new(config.ElevatorDistributor)
	*tempElev = elevatorDistributorInit(newElevator.ID)
	(*tempElev).Behaviour = newElevator.Behaviour
	(*tempElev).Direction = newElevator.Direction
	(*tempElev).Floor = newElevator.Floor
	
	for floor := range tempElev.Requests{
		for button := range tempElev.Requests[floor]{
			tempElev.Requests[floor][button] = newElevator.Requests[floor][button]
		}
	}
	*elevators = append(*elevators, tempElev)
}


// Extracts a new order from the elevator
func confirmedNewOrder(elev *config.ElevatorDistributor) *config.Requests{
	for floor := range elev.Requests {
		for button := 0 ; button < len(elev.Requests[floor]); button++{
			if elev.Requests[floor][button] == config.Order{
				elev.Requests[floor][button] = config.Confirmed 
				tempOrder := new(config.Requests)
				*tempOrder = config.Requests{
					Floor: floor,
					Button: config.ButtonType(button)}
					return tempOrder
				}
			}
		}
		return nil
	}
	

// Set all lights in the elevators according to the requests
func setAllLights(elevators []*config.ElevatorDistributor, elevatorID int) {
	for button := 0 ; button < config.NumButtons -1; button++{
		for floor := 0 ; floor < config.NumFloors ; floor++{
			isLight := false
			for _, elev := range elevators{
				
				if elev.Requests[floor][button] == config.Confirmed{
					isLight = true
				}
			}
			elevio.SetButtonLamp(elevio.ButtonType(button), floor, isLight)
		}
	}
	for floor := 0; floor < config.NumFloors; floor++{
		for _,elev := range elevators{
			if elev.ID == strconv.Itoa(elevatorID) && elev.Requests[floor][elevio.BT_Cab] == config.Confirmed{
				elevio.SetButtonLamp(elevio.BT_Cab, floor, true)
			}
		}
		}
	}
	



