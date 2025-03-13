package masterchecks

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/message"
	"fmt"
	"time"
)

func ShouldKillMaster(elev elevator.Elevator) bool {
	myRank := elev.BackupInfo.RankMap[elev.ID]
	for ID, rank := range elev.BackupInfo.RankMap {
		if myRank <= rank {
			continue
		} else if elev.Connections[ID] && elev.ID == elev.BackupInfo.MasterID {
			fmt.Println("my rank: ", myRank, " and their rank: ", rank)
			fmt.Println("master.ID: ", elev.BackupInfo.MasterID)
			fmt.Println("elev.ID: ", elev.ID)
			fmt.Println(elev.Connections[ID])

			fmt.Println("trying to kill master")
			elev.BackupInfo.MasterID = ""
			return true

		}
	}
	return false
}

// var master_chan = make(chan Elevator.MasterMessage)
func CheckMaster(master_chan chan message.MasterMessage) bool {
	initTimer := time.NewTimer(time.Duration(time.Second))
	for {
		select {
		case <-master_chan:
			fmt.Println("found master")

			return false
		case <-initTimer.C:

			fmt.Println("init timer timed out")
			return true
		default:
			fmt.Println("nothing")
		}
	}
}
