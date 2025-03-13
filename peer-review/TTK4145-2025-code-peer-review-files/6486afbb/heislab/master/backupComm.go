package master

import (
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/bcast"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/peers"
)

/* 
backupsTx transmitts calls read from the callsToBackupsCh channel to the backups
*/
func backupsTx(callsToBackupsCh <-chan Calls, initCalls BackupCalls) {
	masterCallsTx := make(chan BackupCalls)
	go bcast.Transmitter(config.MasterCallsPort, masterCallsTx)
	calls := initCalls
	for {
		select {
		case calls.Calls = <-callsToBackupsCh:
			masterCallsTx <- calls
		case <-time.After(config.MasterMessagePeriodSeconds):
			masterCallsTx <- calls
		}
	}

}


/*
aliveBackupsRx listens the BackupsUpdatePort (from config) and if a backup is lost or reconnected, the function sends a list of the aliveBackups to the aliveBackupsCh. 
*/
func aliveBackupsRx(aliveBackupsCh chan<- []string) {
	backupsUpdateCh := make(chan peers.PeerUpdate)
	go peers.Receiver(config.BackupsUpdatePort, backupsUpdateCh)
	var aliveBackups []string
	for {
		a := <-backupsUpdateCh
		fmt.Printf("Backups update:\n")
		fmt.Printf("  Backups:    %q\n", a.Peers)
		fmt.Printf("  New:        %q\n", a.New)
		fmt.Printf("  Lost:       %q\n", a.Lost)
		aliveBackups = a.Peers
		if len(a.Lost) != 0 || a.New != "" {
			aliveBackupsCh <- aliveBackups
		}
	}
}

/*
backupAckRx starts goroutines to manage backup synchronization. It looks for other masters, tracks the status of alive backups, and sends call assignments to backups as needed.

This routine handles acknowledgments from alive backups, ensuring that all backups are synchronized with the current set of calls before turning the button lights on. 
It also manages the reassignment of calls when necessary.
*/
func backupAckRx(
	callsUpdateCh <-chan UpdateCalls,
	callsToAssignCh chan<- AssignCalls,
	initCalls BackupCalls,
) {
	Id := initCalls.Id

	otherMasterCallsCh := make(chan BackupCalls)
	aliveBackupsCh := make(chan []string)
	callsToBackupsCh := make(chan Calls)
	backupCallsRx := make(chan BackupCalls)

	go bcast.Receiver(config.BackupsCallsPort, backupCallsRx)
	go lookForOtherMasters(otherMasterCallsCh, Id)
	go aliveBackupsRx(aliveBackupsCh)
	go backupsTx(callsToBackupsCh, initCalls)

	var aliveBackups []string
	var acksReceived [config.N_ELEVATORS]bool
	calls := initCalls.Calls
	wantReassignment := false

mainLoop:
	for {
		select {
		case callsUpdate := <-callsUpdateCh:
			if callsUpdate.AddCall {
				calls = union(calls, callsUpdate.Calls)
			} else {
				calls = removeCalls(calls, callsUpdate.Calls)
			}
			callsToBackupsCh <- calls
			wantReassignment = true
			for i := range acksReceived {
				acksReceived[i] = false
			}
			acksReceived[Id] = true
		default:
		}

		select {
		case a := <-backupCallsRx: // set ack for backup if it has the same calls
			if a.Calls == calls && !acksReceived[a.Id] {
				fmt.Println("new backup state from", a.Id)
				acksReceived[a.Id] = true
			}
		default:
		}

		select {
		case aliveBackups = <-aliveBackupsCh:
			wantReassignment = true
		default:
		}

		select {
		case otherMasterCalls := <-otherMasterCallsCh:
			if otherMasterCalls.Id < Id && isCallsSubset(calls, otherMasterCalls.Calls) {
				fmt.Println("find a better way to restart the program")
				os.Exit(42) // intentionally crashing, program restarts automatically when exiting with code 42
			} else if otherMasterCalls.Id > Id {
				calls = union(calls, otherMasterCalls.Calls)
				callsToBackupsCh <- calls
				wantReassignment = true
			} else {
				fmt.Println("couldn't end master phase: other master has not accepted our calls")
			}
		default:
		}

		for _, backup := range aliveBackups { // if some alive backups havent given ack, continue main loop
			i, _ := strconv.Atoi(backup)
			if !acksReceived[i] {
				continue mainLoop
			}
		}
		if wantReassignment {
			fmt.Println("BC: Sending calls")
			var AliveElevators [config.N_ELEVATORS]bool
			for _, elev := range aliveBackups {
				idx, err := strconv.Atoi(elev)
				if err != nil {
					panic("BC got weird aliveElev")
				}
				AliveElevators[idx] = true
			}
			AliveElevators[Id] = true
			callsToAssignCh <- AssignCalls{Calls: calls, AliveElevators: AliveElevators}
			wantReassignment = false
		}
	}
}

// returns true if calls1 is a subset of calls2
func isCallsSubset(calls1 Calls, calls2 Calls) bool {
	for i := range config.N_ELEVATORS {
		for j := range config.N_FLOORS {
			if calls1.CabCalls[i][j] && !calls2.CabCalls[i][j] {
				return false
			}
		}
	}
	for i := range config.N_FLOORS {
		for j := range config.N_BUTTONS - 1 {
			if calls1.HallCalls[i][j] && !calls2.HallCalls[i][j] {
				return false
			}
		}
	}
	return true
}

// returns the union of the calls in calls1 and calls2
func union(calls1 Calls, calls2 Calls) Calls {
	var unionCalls Calls
	for i := range config.N_ELEVATORS {
		for j := range config.N_FLOORS {
			unionCalls.CabCalls[i][j] = calls1.CabCalls[i][j] || calls2.CabCalls[i][j]
		}
	}
	for i := range config.N_FLOORS {
		for j := range config.N_BUTTONS - 1 {
			unionCalls.HallCalls[i][j] = calls1.HallCalls[i][j] || calls2.HallCalls[i][j]
		}
	}
	return unionCalls
}

// returns the set difference between calls and removedCalls
func removeCalls(calls Calls, removedCalls Calls) Calls {
	updatedCalls := calls

	for i := range config.N_ELEVATORS {
		for j := range config.N_FLOORS {
			updatedCalls.CabCalls[i][j] = calls.CabCalls[i][j] && !removedCalls.CabCalls[i][j]
		}
	}
	for i := range config.N_FLOORS {
		for j := range config.N_BUTTONS - 1 {
			updatedCalls.HallCalls[i][j] = calls.HallCalls[i][j] && !removedCalls.HallCalls[i][j]
		}
	}
	return updatedCalls
}
