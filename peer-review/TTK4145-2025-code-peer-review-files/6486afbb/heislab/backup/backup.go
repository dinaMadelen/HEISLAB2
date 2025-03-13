package backup

import (
	"fmt"
	"slices"
	"strconv"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/master"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/bcast"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/peers"
	"github.com/Kirlu3/Sanntid-G30/heislab/slave"
)

/*
	The entire backup system run in one goroutine.

The routine listens to the master's UDP broadcasts and responds with the updated worldview.
If the backup loses connection with the master, it will transition to the master phase with its current worldview.
A large portion of the backup code are pretty prints of updates to peer lists.
*/
func Backup(
	id string,
	masterToSlaveOfflineCh chan<- [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool,
	slaveToMasterOfflineCh <-chan slave.EventMessage,
) {
	masterUpdateCh := make(chan peers.PeerUpdate)
	backupsUpdateCh := make(chan peers.PeerUpdate)
	backupsTxEnable := make(chan bool)
	backupCallsTx := make(chan master.BackupCalls)
	masterCallsRx := make(chan master.BackupCalls)

	go peers.Receiver(config.MasterUpdatePort, masterUpdateCh)

	go peers.Transmitter(config.BackupsUpdatePort, id, backupsTxEnable)
	go peers.Receiver(config.BackupsUpdatePort, backupsUpdateCh)

	go bcast.Transmitter(config.BackupsCallsPort, backupCallsTx)

	go bcast.Receiver(config.MasterCallsPort, masterCallsRx)

	fmt.Println("Backup Started: ", id)
	var backupsUpdate peers.PeerUpdate
	var masterUpdate peers.PeerUpdate
	var calls master.BackupCalls
	idInt, err := strconv.Atoi(id)
	if err != nil {
		panic("backup received invalid id")
	}
	calls.Id = idInt

	masterUpgradeCooldown := time.NewTimer(1 * time.Second)
	for {
		select {
		case c := <-masterCallsRx:
			if len(masterUpdate.Peers) > 0 && strconv.Itoa(c.Id) == masterUpdate.Peers[0] {
				calls.Calls = c.Calls
			} else {
				fmt.Println("received a message from not the master")
			}

		case backupsUpdate = <-backupsUpdateCh:
			fmt.Printf("Backups update:\n")
			fmt.Printf("  Backups:    %q\n", backupsUpdate.Peers)
			fmt.Printf("  New:        %q\n", backupsUpdate.New)
			fmt.Printf("  Lost:       %q\n", backupsUpdate.Lost)

		case masterUpdate = <-masterUpdateCh:
			fmt.Printf("Master update:\n")
			fmt.Printf("  Masters:    %q\n", masterUpdate.Peers)
			fmt.Printf("  New:        %q\n", masterUpdate.New)
			fmt.Printf("  Lost:       %q\n", masterUpdate.Lost)

		case <-time.After(time.Second * 2):
			fmt.Println("backup select blocked for 2 seconds. this should only happen if there are no masters, maybe this is too short?")
		}
		backupCallsTx <- calls
		if len(masterUpdate.Peers) == 0 && len(backupsUpdate.Peers) != 0 && slices.Min(backupsUpdate.Peers) == id && func() bool {
			select {
			case <-masterUpgradeCooldown.C:
				return true
			default:
				return false
			}
		}() {
			backupsTxEnable <- false
			master.Master(calls, masterToSlaveOfflineCh, slaveToMasterOfflineCh)
			panic("the master phase should never return")
		}
	}
}
