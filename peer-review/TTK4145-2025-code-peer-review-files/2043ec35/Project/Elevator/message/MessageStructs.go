package message

import (
	"elevproj/Elevator/elevator"
	"elevproj/Elevator/elevio"
	"elevproj/config"
)

type MasterMessage struct {
	ID                      string
	RankMap                 map[string]int
	FullHallRequests        [config.N_floors][2]bool
	DistributedHallRequests map[string][config.N_floors][2]bool
	FullCabRequests         map[string][config.N_floors]bool
	SetAllLights            bool
}

func DeepCopyMasterMessage(message MasterMessage) MasterMessage {
	var copy MasterMessage
	copy.ID = message.ID
	copy.RankMap = message.RankMap
	copy.FullHallRequests = message.FullHallRequests
	copy.DistributedHallRequests = message.DistributedHallRequests
	copy.FullCabRequests = message.FullCabRequests
	copy.SetAllLights = message.SetAllLights

	return copy
}

// Er Report det beste navnet?
type ReportMessage struct {
	ID     string
	Report ElevatorReport
}

type ElevatorReport struct {
	Floor     int
	Dirn      elevio.MotorDirection
	Requests  [config.N_floors][config.N_buttons]bool
	Behaviour elevator.ElevatorBehaviour

	BackupInfo elevator.BackupInfo
}

func DeepCopyReportMessage(message ReportMessage) ReportMessage {
	var copy ReportMessage
	copy.ID = message.ID
	copy.Report = message.Report

	return copy
}

func UpdateReport(elev elevator.Elevator) ReportMessage {
	var newReportMessage ReportMessage

	newReportMessage.ID = elev.ID
	newReportMessage.Report.Floor = elev.LatestFloor
	newReportMessage.Report.Dirn = elev.Dirn
	newReportMessage.Report.Requests = elev.Requests
	newReportMessage.Report.Behaviour = elev.Behaviour
	newReportMessage.Report.BackupInfo.RankMap = elev.BackupInfo.RankMap
	newReportMessage.Report.BackupInfo.MasterID = elev.BackupInfo.MasterID
	newReportMessage.Report.BackupInfo.FullHallRequests = elev.BackupInfo.FullHallRequests
	newReportMessage.Report.BackupInfo.FullCabRequests = elev.BackupInfo.FullCabRequests
	newReportMessage.Report.BackupInfo.DistributedHallRequests = elev.BackupInfo.DistributedHallRequests

	return newReportMessage
}

type BtnPress struct {
	ID  string
	Btn elevio.ButtonEvent
}
