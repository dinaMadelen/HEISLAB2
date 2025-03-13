package master

import (
	"fmt"
	"strconv"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/peers"
	"github.com/Kirlu3/Sanntid-G30/heislab/slave"
)

func Master(
	initCalls BackupCalls,
	masterToSlaveOfflineCh chan<- [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool,
	slaveToMasterOfflineCh <-chan slave.EventMessage,
) {
	fmt.Println(initCalls.Id, "entered master mode")

	callsUpdateCh := make(chan UpdateCalls, 2)
	callsToAssignCh := make(chan AssignCalls)

	stateUpdateCh := make(chan slave.Elevator)
	assignmentsToSlaveCh := make(chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool)
	assignmentsToSlaveReceiverCh := make(chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool, 2)

	masterTxEnable := make(chan bool)

	go peers.Transmitter(config.MasterUpdatePort, strconv.Itoa(initCalls.Id), masterTxEnable)

	go backupAckRx(callsUpdateCh, callsToAssignCh, initCalls)
	go assignOrders(stateUpdateCh, callsToAssignCh, assignmentsToSlaveCh, assignmentsToSlaveReceiverCh)

	go receiveMessagesFromSlaves(stateUpdateCh, callsUpdateCh, assignmentsToSlaveReceiverCh, slaveToMasterOfflineCh)
	go sendMessagesToSlaves(assignmentsToSlaveCh, masterToSlaveOfflineCh)

	// the program is crashed and restarted when it should go back to backup mode
	select {}
}
