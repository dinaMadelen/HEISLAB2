package hub

import (
	"hub/hra"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo/hra"
)

const BroadcastingIP = "255.255.255.255"

const (
	Idle = iota
	Backup
	Active
)

var (
	N_floors = 4
	N_elevators = 3
)

type Hub struct {
	State      int
	WVs        [3]WorldView
	IP		   string
	BackupIP   string
}

type WorldView struct {
	SenderElevState hra.HRAElevState
	HallRequests    [4][2]bool
	IP              string
	LastSeen		time.Time   
}

type Heartbeat struct {
	IP string
	State int
	Instruction int
}

// Copy from elevio, could instead import package
type ButtonType int

const (
	BT_HallUp   ButtonType = 0
	BT_HallDown            = 1
	BT_Cab                 = 2
)

type ButtonEvent struct {
	Floor  int
	Button ButtonType
}

type MessageType int

const (
	MsgTypeHeartbeat MessageType = iota
	MsgTypeWorldview
	MsgTypeButtonReq
	MsgTypeButtonOrder
)

var messageType = map[MessageType]string{ //Defining the ports of the different kinds of messages
	MsgTypeHeartbeat:   "65000",
	MsgTypeWorldview:   "65001",
	MsgTypeButtonReq:   "65002",
	MsgTypeButtonOrder: "65003",

}
func Initialize() {
	
}
