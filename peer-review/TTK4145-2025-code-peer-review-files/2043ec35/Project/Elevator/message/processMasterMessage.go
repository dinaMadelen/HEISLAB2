package message

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/elevio"
	"fmt"
)

// updates elevatorobject.
func ProcessMasterMessage(message MasterMessage, elev elevator.Elevator) (elevator.Elevator, elevio.ButtonEvent) {
	var newBtn elevio.ButtonEvent
	newBtn.Floor = -1

	newElevator := elevator.DeepCopyElevator(elev)
	newElevator.BackupInfo.MasterID = message.ID

	//updating rankmap
	for ID, rank := range message.RankMap {
		newElevator.BackupInfo.RankMap[ID] = rank
	}

	//updating fullHallRequests
	for i := range message.FullHallRequests {
		for j, request := range message.FullHallRequests[i] {
			newElevator.BackupInfo.FullHallRequests[i][j] = request
		}
	}

	//updating DistributedHallRequests
	for ID, _ := range message.DistributedHallRequests {
		masterDistributed := message.DistributedHallRequests[ID]
		elevDistributed := newElevator.BackupInfo.DistributedHallRequests[newElevator.ID]
		for i := range masterDistributed {
			for j, requestHere := range masterDistributed[i] {
				//fmt.Println(requestHere)
				if ID == newElevator.ID && requestHere && !elevDistributed[i][j] {
					//fmt.Println("elevators distributed requests: ", elevDistributed[i][j])
					newBtn = elevio.ButtonEvent{i, elevio.ButtonType(j)}
				}
				elevDistributed[i][j] = requestHere
			}
		}
		newElevator.BackupInfo.DistributedHallRequests[ID] = elevDistributed
	}

	//updating fullCabRequest
	for ID, requests := range message.FullCabRequests {
		cabRequests := newElevator.BackupInfo.FullCabRequests[ID]
		for i, requestHere := range requests {
			if requestHere && !cabRequests[i] {
				fmt.Println("found cab request")
				if ID == newElevator.ID {
					newBtn = elevio.ButtonEvent{i, 2}
				}
			}
			cabRequests[i] = requestHere
		}
		newElevator.BackupInfo.FullCabRequests[ID] = cabRequests
	}

	return newElevator, newBtn
}
