package elevio

import (
	"net"
	"sync"
	"time"
)

// --- HARDWARE INTERACTION CONSTANTS --- //

const PollRate = 20 * time.Millisecond

// --- HARDWARE INTERACTION STRUCTS --- //

type Ele struct {
	ID        int
	Conn      net.Conn
	NumFloors int
	Mtx       sync.Mutex
}

type MotorDirection int

const (
	MD_Up   MotorDirection = 1
	MD_Down                = -1
	MD_Stop                = 0
)

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

type OptimalButtonEvent struct {
	Floor      int
	Button     ButtonType
	ElevatorID int
}
