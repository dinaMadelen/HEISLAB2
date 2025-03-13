package mastermessageprocessing

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/message"
	"elevproj/Master/masterObject"
	"elevproj/Master/requestDistribution"
	"fmt"
)

// må denne returnere alle verdiene den endrer på?
// legg til at returnerer bool?
// tror ikke det er så bra å ta inn master som pointer, vi burde nok heller returnere
func ProcessMessage(message message.ReportMessage, master masterobject.Master) (bool, masterobject.Master) {
	newMaster := masterobject.DeepCopyMaster(master)
	id := message.ID
	newMaster.FullHallRequests = requestdistribution.CheckFulfilledRequests(message, newMaster.FullHallRequests)
	distributed := newMaster.DistributedHallRequests[id]
	cabRequests := newMaster.FullCabRequests[id]
	//kanskje lage denne penere
	for i := range distributed {
		for j := range message.Report.Requests[i] {
			if j == 2 {
				cabRequests[i] = message.Report.Requests[i][j]
			} else {
				distributed[i][j] = message.Report.Requests[i][j]
			}
		}
		newMaster.FullCabRequests[id] = cabRequests
	}
	if message.Report.Behaviour == elevator.EB_stop || message.Report.Behaviour == elevator.EB_obstruct {
		if _, exists := newMaster.ElevatorReports[id]; exists {
			delete(newMaster.ElevatorReports, id)
		}
	} else {
		newMaster.ElevatorReports[id] = message.Report
	}
	newMaster.DistributedHallRequests[id] = distributed
	//newMaster.FullCabRequests[id] = message.Report.SystemInfo.FullCabRequests[id]

	HRAready := true

	masterFullHallRequests := newMaster.FullHallRequests
	for _, report := range newMaster.ElevatorReports {
		for i, floor := range report.BackupInfo.FullHallRequests {
			for j, btnPress := range floor {
				if btnPress != masterFullHallRequests[i][j] {
					HRAready = false
				}
			}
		}
	}
	return HRAready, newMaster
}

func ProcessButton(btnMessage message.BtnPress, master masterobject.Master) masterobject.Master {
	newMaster := masterobject.DeepCopyMaster(master)
	btnType := btnMessage.Btn.Button
	if btnType == 2 {
		cabRequests := newMaster.FullCabRequests[btnMessage.ID]
		cabRequests[btnMessage.Btn.Floor] = true
		newMaster.FullCabRequests[btnMessage.ID] = cabRequests
	} else {
		fmt.Println("buttontype: ", btnType)
		//fmt.Println("adding button to master fullRequest")
		newMaster.FullHallRequests[btnMessage.Btn.Floor][btnType] = true
	}
	return newMaster

}

func GetBackupInfo(master masterobject.Master, report message.ReportMessage) masterobject.Master {
	master.ElevatorReports[report.ID] = report.Report
	master.RankMap = report.Report.BackupInfo.RankMap
	master.FullHallRequests = report.Report.BackupInfo.FullHallRequests
	master.DistributedHallRequests = report.Report.BackupInfo.DistributedHallRequests
	master.FullCabRequests = report.Report.BackupInfo.FullCabRequests

	return master
}

func UpdateMessage(m message.MasterMessage, master masterobject.Master) message.MasterMessage {
	m.ID = master.ID
	m.RankMap = master.RankMap
	m.FullHallRequests = master.FullHallRequests
	m.DistributedHallRequests = master.DistributedHallRequests
	m.FullCabRequests = master.FullCabRequests

	newMessage := message.DeepCopyMasterMessage(m)
	return newMessage
}
