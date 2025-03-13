package main

import (
	"elevatorsystem/Network-go/network/bcast"
	"elevatorsystem/Network-go/network/localip"
	"elevatorsystem/Network-go/network/peers"
	"elevatorsystem/assignment"
	"elevatorsystem/constants"
	"elevatorsystem/distribution"
	"elevatorsystem/single-elevator/Driver-go/elevio"
	"elevatorsystem/single-elevator/elevatorLogic"
	"elevatorsystem/single-elevator/fsm"
	"elevatorsystem/single-elevator/timer"
	"flag"
	"fmt"
	"os"
	"time"
)

type bcast_msg_orderStatesAllElevators struct {
	orderStatesAllElevators map[string][constants.NUM_FLOORS][constants.NUM_BUTTONS]int
	senderID                string
}

func updateOrAppendElevatorData(elevatorDataList []elevatorLogic.Elevator, newData elevatorLogic.Elevator) []elevatorLogic.Elevator {
	for i, data := range elevatorDataList {
		if data.ElevatorID == newData.ElevatorID {
			elevatorDataList[i] = newData
			return elevatorDataList
		}
	}
	return append(elevatorDataList, newData)
}

func main() {
	// Initialize
	elevio.Init("localhost: 15657", constants.NUM_FLOORS)
	elevator := fsm.GetElevator()

	orderStatesAllElevators := make(map[string][constants.NUM_FLOORS][constants.NUM_BUTTONS]int)
	elevatorDataMessage := []elevatorLogic.Elevator{}
	peerAliveList := []string{}

	orderStatesAllElevators_msg := bcast_msg_orderStatesAllElevators{
		orderStatesAllElevators: orderStatesAllElevators,
		senderID:                elevator.ElevatorID,
	}

	// Handle the case where the elevator starts between floors
	if elevio.GetFloor() == -1 {
		fsm.OnInitBetweenFloors(elevator)
	}

	// Get the id of this peer from the command line or use a default id
	var id string
	flag.StringVar(&id, "id", "", "id of this peer")
	flag.Parse()

	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		id = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
	}

	// Channels and go routines for sending and receiving peer updates
	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	go peers.Transmitter(15647, id, peerTxEnable)
	go peers.Receiver(15647, peerUpdateCh)

	// Channels and go routines for broadcasting and receiving elevator data
	elevatorTx := make(chan elevatorLogic.Elevator)
	elevatorRx := make(chan elevatorLogic.Elevator)
	go bcast.Transmitter(16569, elevatorTx)
	go bcast.Receiver(16569, elevatorRx)

	// Channels for broadcasting and receiving orderStatesAllElevators
	orderStatesAllElevatorsTx := make(chan bcast_msg_orderStatesAllElevators)
	orderStatesAllElevatorsRx := make(chan bcast_msg_orderStatesAllElevators)
	go bcast.Transmitter(16568, orderStatesAllElevatorsTx)
	go bcast.Receiver(16568, orderStatesAllElevatorsRx)

	// Broadcast order states for all elevators and elevator data
	go func() {
		for {
			elevatorTx <- *elevator
			orderStatesAllElevatorsTx <- orderStatesAllElevators_msg
			time.Sleep(constants.BCAST_SEND_RATE)
		}
	}()

	// Channels and go routines for getting data from elevator
	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	// Channels and go routine for the door timer
	timer_timeout := make(chan bool)
	go timer.PollTimer(timer_timeout)

	// Obstruction logic
	obstruction := false
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for {
		select {

		case a := <-peerUpdateCh:
			peerAliveList = a.Peers

		case a := <-elevatorRx:
			elevatorDataMessage = updateOrAppendElevatorData(elevatorDataMessage, a)

		case a := <-orderStatesAllElevatorsRx:
			distribution.UpdateOrderStatesAllElevators(&orderStatesAllElevators, a.senderID, a.orderStatesAllElevators)

		case a := <-drv_buttons:
			if a.Button != elevio.BT_Cab {
				fromAssign := assignment.Assign(peerAliveList, elevatorDataMessage, a)
				// Iterate through fromAssign to find the element that is true
				for ID, orders := range fromAssign {
					for floor := 0; floor < constants.NUM_FLOORS; floor++ {
						for button := 0; button < 2; button++ {
							if orders[floor][button] {
								temp := orderStatesAllElevators[ID]
								temp[floor][button] = 1
								orderStatesAllElevators[ID] = temp
							}
						}
					}
				}
			} else {
				temp := orderStatesAllElevators[elevator.ElevatorID]
				temp[a.Floor][a.Button] = 1
				orderStatesAllElevators[elevator.ElevatorID] = temp
			}
			fmt.Printf("Button pressed: %+v\n", a)
			if !elevator.Orders[a.Floor][a.Button] {
				fsm.OnOrderButtonPress(elevator, a)
			}
		// Not implemented yet
		// case a := <-distribution.NewOrder:
		// 	if !e.OrderList[a.Floor][a.Button] {
		// 		fsm.OnOrderButtonPress(e, a)
		// 	}
		case a := <-drv_floors:
			fmt.Printf("Floor arrival: %+v\n", a)
			if a != -1 && a != elevator.LastKnownFloor {
				fsm.OnFloorArrival(elevator, a)
			}

		case a := <-timer_timeout:
			fmt.Printf("Timer timedout: %+v\n", a)
			timer.TimerStop()
			fsm.OnDoorTimeout(elevator)

		case a := <-drv_obstr:
			fmt.Printf("Obstruction: %+v\n", a)
			obstruction = a
		// If the obstruction is true, postpone the endtime of the door-timer
		case <-ticker.C:
			if obstruction {
				timer.AddTime(float64(2))
				fmt.Print("a=true, Updating endtime\n")
			}
		}
	}
}
