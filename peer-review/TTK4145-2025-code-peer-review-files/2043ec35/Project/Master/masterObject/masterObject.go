package masterobject

import (
	"elevproj/Elevator/message"
	"elevproj/Network/localip"
	"elevproj/config"
)

type Master struct {
	ID                      string
	ElevatorReports         map[string]message.ElevatorReport // on form IPaddress:elevatorstate
	Connections             map[string]bool                   // oppdaterer vi dette når vi mister connection?
	RankMap                 map[string]int
	FullHallRequests        [config.N_floors][2]bool
	DistributedHallRequests map[string][config.N_floors][2]bool
	FullCabRequests         map[string][config.N_floors]bool
}

func MakeEmptyMasterObject() Master {
	IP, _ := localip.LocalIP()
	var master Master
	master.ID = IP
	master.ElevatorReports = make(map[string]message.ElevatorReport)
	master.FullCabRequests = make(map[string][config.N_floors]bool)
	master.RankMap = make(map[string]int)
	master.DistributedHallRequests = make(map[string][config.N_floors][2]bool)
	master.Connections = make(map[string]bool)

	return master
}

func DeepCopyMaster(oldMaster Master) Master {
	var newMaster Master
	newMaster.ID = oldMaster.ID
	newMaster.Connections = oldMaster.Connections
	newMaster.ElevatorReports = oldMaster.ElevatorReports
	newMaster.FullCabRequests = oldMaster.FullCabRequests
	newMaster.FullHallRequests = oldMaster.FullHallRequests
	newMaster.DistributedHallRequests = oldMaster.DistributedHallRequests
	newMaster.RankMap = oldMaster.RankMap

	return newMaster
}
