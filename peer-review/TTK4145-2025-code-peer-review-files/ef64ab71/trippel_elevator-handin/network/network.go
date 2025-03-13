package network

import (
	"fmt"
	"strconv"
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/delegateOrders"
	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
	"github.com/Eirik-a-Johansen/trippel_elevator/network/bcast"
	"github.com/Eirik-a-Johansen/trippel_elevator/network/peers"
)

/*
This module is responsible for the communication between the elevators

It has two ports for communication: one for checking wich elevators are connected and one for sending worldview
The code for transmitting and reciving is from ttk 4145 github. They are removed for hand in

We are using udp

*/

type Message struct {
	Elevators [elevator.NumberOfElevators]elevator.Elevator
	Delegated [driver.N_Floors][driver.N_Buttons + (elevator.NumberOfElevators - 1)]int
	ID        int
}

func Network(e *elevator.Elevator) {

	id := strconv.Itoa(elevator.LocalElevator.ID)

	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	outgoingMessage := make(chan Message)
	IncomingMessage := make(chan Message)

	go peers.Transmitter(30010, id, peerTxEnable)
	go peers.Receiver(30010, peerUpdateCh)

	go bcast.Transmitter(16569, outgoingMessage)
	go bcast.Receiver(16569, IncomingMessage)

	go func() {
		var message Message

		for {
			elevator.Mutex.Lock()
			elevator.Elevators[e.ID] = *e //update local worldview in message
			message = Message{elevator.Elevators, elevator.Delegated, e.ID}
			elevator.Mutex.Unlock()

			outgoingMessage <- message

			time.Sleep(10 * time.Millisecond)
		}
	}()

	fmt.Println("Started")
	connected := [elevator.NumberOfElevators]bool{}
	for {
		select {
		case peerUpdate := <-peerUpdateCh:

			elevator.Mutex.Lock()
			connected = [elevator.NumberOfElevators]bool{}

			fmt.Println("Peer update: ", peerUpdate.Peers, peerUpdate.Lost)

			intPeerList := convertStringToIntList(peerUpdate.Peers)
			if e.ID == intPeerList[0] { //lowest id is master in system, this is set in initialize
				e.IsMaster = true
				elevator.Elevators[e.ID].IsMaster = true
			} else {
				e.IsMaster = false
				elevator.Elevators[e.ID].IsMaster = false
			}

			for i := 0; i < len(peerUpdate.Peers); i++ {
				connected[intPeerList[i]] = true

			}

			e.OnlineElevators = connected //overview of connected elevators

			lostElevatorsIntList := convertStringToIntList(peerUpdate.Lost)

			for i := 0; i < len(peerUpdate.Lost); i++ {
				delegateOrders.UnassignOrders(lostElevatorsIntList[i]) //redelegate orders from disconnected elevators
			}
			elevator.Mutex.Unlock()

		case receivedMessage := <-IncomingMessage:
			elevator.Mutex.Lock()

			idMaster := -1
			for i, val := range e.OnlineElevators {
				if val {
					idMaster = i
					break
				}
			}

			if idMaster == receivedMessage.ID {
				updateMyOrders(e, receivedMessage) //only master can delegate orders
			}

			for i := 0; i < elevator.NumberOfElevators; i++ { //update worldview on other elevator
				if i == e.ID {
					continue
				}
				elevator.Elevators[i] = receivedMessage.Elevators[i]
			}
			updateDelegatedOrders(e, receivedMessage)

			if !receivedMessage.Elevators[receivedMessage.ID].Functional {
				delegateOrders.UnassignOrders(receivedMessage.ID)
			}

			elevator.Mutex.Unlock()

		}
	}
}

func convertStringToIntList(stringList []string) []int {
	intList := make([]int, len(stringList))
	for i, str := range stringList {
		num, err := strconv.Atoi(str)
		if err != nil {
			fmt.Println("Error converting", err)
		}
		intList[i] = num
	}
	return intList
}

func updateMyOrders(e *elevator.Elevator, recievedMessage Message) {
	for i := 0; i < driver.N_Floors; i++ {
		for j := 0; j < driver.N_Buttons+(elevator.NumberOfElevators-1); j++ {
			if j >= driver.N_Buttons-1 {
				if recievedMessage.Delegated[i][j] == e.ID && elevator.Delegated[i][j] != -2 && e.Orders[i][j].Value != 3 {
					e.MyOrders[i][j-e.ID] = 1
				}
				continue
			}
			if recievedMessage.Delegated[i][j] == e.ID && elevator.Delegated[i][j] != -2 && e.Orders[i][j].Value != 3 {
				e.MyOrders[i][j] = 1
			}
		}
	}
}

func updateDelegatedOrders(e *elevator.Elevator, recievedMessage Message) {
	for i := 0; i < elevator.NumberOfElevators; i++ {
		if i == e.ID {
			continue
		}
		for i := 0; i < driver.N_Floors; i++ {
			for j := 0; j < driver.N_Buttons+(elevator.NumberOfElevators-1); j++ {
				if j >= driver.N_Buttons-1 {
					if j-recievedMessage.ID == driver.N_Buttons-1 && recievedMessage.Delegated[i][j] == -2 {
						elevator.Delegated[i][j] = -1
					}
					continue
				}
				if recievedMessage.Delegated[i][j] == -2 {
					elevator.Delegated[i][j] = -1
				}
			}
		}
	}
}
