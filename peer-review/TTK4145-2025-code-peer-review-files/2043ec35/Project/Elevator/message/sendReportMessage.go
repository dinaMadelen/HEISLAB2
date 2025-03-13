package message

import "elevproj/Elevator/elevator"

//"fmt"

func SendMessageToMaster(elev elevator.Elevator, messageTx_chan chan ReportMessage, elevTxenable_chan chan bool) {
	//report:= ReportMessage
	report := UpdateReport(elev)
	reportCopy := DeepCopyReportMessage(report)
	elevTxenable_chan <- true
	messageTx_chan <- reportCopy
}
