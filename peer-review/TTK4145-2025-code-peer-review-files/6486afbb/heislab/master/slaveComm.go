package master

import (
	"fmt"
	"slices"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/driver-go/elevio"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/bcast"
	"github.com/Kirlu3/Sanntid-G30/heislab/slave"
)

/*
receiveMessagesFromSlaves handles updates from the slaves and updates the state of the elevators accordingly. The routine also either add or remove calls dependent on the type of event in the update from the slaves.
*/
func receiveMessagesFromSlaves(
	stateUpdateCh chan<- slave.Elevator,
	callsUpdateCh chan<- UpdateCalls,
	assignmentsToSlaveReceiver <-chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool,
	slaveToMasterOfflineCh <-chan slave.EventMessage,
) {

	slaveRx := make(chan slave.EventMessage)
	go receiveUniqueMessages(slaveRx)

	go func() {
		for msg := range slaveToMasterOfflineCh {
			slaveRx <- msg
		}
	}()
	var assignments [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool
	for {
		select {
		case update := <-slaveRx:
			fmt.Println("ST: Received new message")
			fmt.Println(update)
			switch update.Event {
			case slave.Button:
				//stateUpdateCh <- update.Elevator //can no longer do this with how button presses are set up
				callsUpdateCh <- makeAddCallsUpdate(update)
			case slave.FloorArrival:
				stateUpdateCh <- update.Elevator
				if update.Elevator.Behaviour == slave.EB_DoorOpen {
					callsUpdateCh <- makeRemoveCallsUpdate(update, assignments)
				}
			case slave.Stuck:
				stateUpdateCh <- update.Elevator
			}
		case assignments = <-assignmentsToSlaveReceiver:
			continue
		}
	}
}

/*
receiveMessageFromSlave listens to the SlaveBasePort+slaveID (from config) for messages from slaves and transmitts the message IDs to the SlaveBasePort+10+slaveID (from config).
*/
func receiveUniqueMessages(slaveRx chan<- slave.EventMessage) {

	//rx channel for receiving messages
	rx := make(chan slave.EventMessage)
	go bcast.Receiver(config.SlaveBasePort, rx)
	//ack channel to send an acknowledgments
	ack := make(chan int)
	go bcast.Transmitter(config.SlaveBasePort+10, ack)

	var msgID []int
	for msg := range rx {
		println("ST: Received message")
		ack <- msg.MsgID
		fmt.Println("ST: Sent Ack", msg.MsgID)
		if !slices.Contains(msgID, msg.MsgID) {
			msgID = append(msgID, msg.MsgID)
			// if we've stored too many IDs, remove the oldest one. 20 is a completely arbitrary number, but leaves room for ~7 messages per slave
			if len(msgID) > 20 {
				msgID = msgID[1:]
			}
			slaveRx <- msg
		}
	}
}

/*
Inputs: EventMessage and an array of the assignments

Output: UpdateCalls struct

Function transforms the inputs to the right output type that is used for handling updates in the calls.
*/
func makeRemoveCallsUpdate(msg slave.EventMessage, assignments [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool) UpdateCalls {
	var callsUpdate UpdateCalls
	callsUpdate.AddCall = false

	callsUpdate.Calls.CabCalls[msg.Elevator.ID][msg.Elevator.Floor] = true
	for btn := range config.N_BUTTONS - 1 {
		if assignments[msg.Elevator.ID][msg.Elevator.Floor][btn] && !msg.Elevator.Requests[msg.Elevator.Floor][btn] {
			callsUpdate.Calls.HallCalls[msg.Elevator.Floor][btn] = true
		}
	}
	return callsUpdate
}

/*
Input: EventMessage

Output: UpdateCalls

Function transforms the input to the right output type that is used for handling updates in the calls.
*/
func makeAddCallsUpdate(msg slave.EventMessage) UpdateCalls {
	var callsUpdate UpdateCalls
	callsUpdate.AddCall = true
	if msg.Btn.Button == elevio.BT_Cab {
		callsUpdate.Calls.CabCalls[msg.Elevator.ID][msg.Btn.Floor] = true
	} else {
		callsUpdate.Calls.HallCalls[msg.Btn.Floor][msg.Btn.Button] = true
	}
	return callsUpdate
}

/*
Handles transmitting the assignments received on the toSlaveCh channel to the slaves on the SlaveBasePort-1 port.
*/
func sendMessagesToSlaves(toSlaveCh chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool,
	masterToSlaveOfflineCh chan<- [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool) {
	tx := make(chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool)
	go bcast.Transmitter(config.SlaveBasePort-1, tx)

	var msg [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool
	for {
		//Gives message frequency
		time.Sleep(time.Millisecond * 5)

		select {
		case msg = <-toSlaveCh:
			fmt.Println("ST: New orders sent")
			fmt.Println(msg)
			tx <- msg
			masterToSlaveOfflineCh <- msg
		default:
			tx <- msg
		}
	}
}
