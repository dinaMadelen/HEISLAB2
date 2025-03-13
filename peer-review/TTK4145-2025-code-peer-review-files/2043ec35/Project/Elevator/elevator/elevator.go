package elevator

import (
	"elevproj/Elevator/elevio"
	"elevproj/config"
	"fmt"
	"time"
)
type ElevatorBehaviour int

const (
	EB_idle     ElevatorBehaviour = 1
	EB_dooropen ElevatorBehaviour = 2
	EB_moving   ElevatorBehaviour = 3
	EB_obstruct ElevatorBehaviour = 4
	EB_stop     ElevatorBehaviour = 5
)

type Elevator struct {
	ID          string
	Case        ElevatorCase
	LatestFloor int
	Dirn        elevio.MotorDirection
	Requests    [config.N_floors][config.N_buttons]bool
	Behaviour   ElevatorBehaviour
	Connections map[string]bool

	//Er dette dårlig navngivning?
	BackupInfo BackupInfo

	DoorOpenDuration time.Duration
}

type ElevatorCase int

const (
	PeerElevator   ElevatorCase = 0
	SingleElevator              = 1
)

type BackupInfo struct {
	MasterID                string
	RankMap                 map[string]int
	FullHallRequests        [config.N_floors][2]bool
	DistributedHallRequests map[string][config.N_floors][2]bool
	FullCabRequests         map[string][config.N_floors]bool
}

func MakeEmptyElevatorObject(IP string) Elevator {
	var elevator Elevator
	elevator.ID = IP
	elevator.Case = PeerElevator
	elevator.DoorOpenDuration = 3 * time.Second
	elevator.Behaviour = EB_idle
	elevator.Dirn = elevio.MD_Stop
	elevator.BackupInfo.RankMap = make(map[string]int)
	elevator.BackupInfo.DistributedHallRequests = make(map[string][config.N_floors][2]bool)
	elevator.BackupInfo.FullHallRequests = [config.N_floors][2]bool{}
	elevator.BackupInfo.FullCabRequests = make(map[string][config.N_floors]bool)
	elevator.Connections = make(map[string]bool)
	elevator.BackupInfo.MasterID = ""

	return elevator
}

func DeepCopyElevator(oldElevator Elevator) Elevator {
	var newElevator Elevator
	newElevator.ID = oldElevator.ID
	newElevator.Case = oldElevator.Case
	newElevator.LatestFloor = oldElevator.LatestFloor
	newElevator.Dirn = oldElevator.Dirn
	newElevator.Requests = oldElevator.Requests
	newElevator.Behaviour = oldElevator.Behaviour
	newElevator.Connections = oldElevator.Connections
	newElevator.DoorOpenDuration = oldElevator.DoorOpenDuration
	newElevator.BackupInfo.RankMap = oldElevator.BackupInfo.RankMap
	newElevator.BackupInfo.FullHallRequests = oldElevator.BackupInfo.FullHallRequests
	newElevator.BackupInfo.MasterID = oldElevator.BackupInfo.MasterID
	newElevator.BackupInfo.DistributedHallRequests = oldElevator.BackupInfo.DistributedHallRequests
	newElevator.BackupInfo.FullCabRequests = oldElevator.BackupInfo.FullCabRequests

	return newElevator
}

func Eb_toString(eb ElevatorBehaviour) string {
	switch eb {
	case EB_idle:
		return "idle"
	case EB_dooropen:
		return "doorOpen"
	case EB_moving:
		return "moving"
	default:
		return ""
	}
}

func Ed_toString(md elevio.MotorDirection) string {
	switch md {
	case 0:
		return "stop"
	case 1:
		return "up"
	case -1:
		return "down"
	default:
		return ""
	}
}
func InitializeElevator(ports string, elevator Elevator) (chan int, chan bool, chan bool, chan elevio.ButtonEvent, int) {
	//creating necessary channels
	var drv_floors = make(chan int)
	var drv_obstr = make(chan bool)
	var drv_stop = make(chan bool)
	var drv_buttons = make(chan elevio.ButtonEvent)
	var newInitFloor int

	//The elevator go routines
	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)

	elevio.Init(ports, int(config.N_floors))
	initFloor := <-drv_floors
	fmt.Println(initFloor)
	for initFloor == -1 {
		initFloor = <-drv_floors
		elevio.SetMotorDirection(1)
	}
	if initFloor != -1 {
		elevio.SetMotorDirection(0)
		elevio.SetFloorIndicator(initFloor)
		newInitFloor = initFloor
	}
	fmt.Println("moving on from init")
	return drv_floors, drv_obstr, drv_stop, drv_buttons, newInitFloor
}

func FindHigherRankConnections(elevator Elevator, myRank int) []int {
	higherRankConnections := []int{}
	for ID, rank := range elevator.BackupInfo.RankMap {
		if elevator.Connections[ID] {
			continue
		} else if !elevator.Connections[ID] {
			if rank < myRank {
				higherRankConnections = append(higherRankConnections, rank)
			}
		}
	}
	return higherRankConnections
}

func SetAllLights(e Elevator) Elevator {
	newElevator := DeepCopyElevator(e)
	for floor := 0; floor < config.N_floors; floor++ {
		for btn := 0; btn < config.N_buttons; btn++ {
			var requestHere bool
			if btn == elevio.BT_Cab {
				requestHere = newElevator.Requests[floor][btn]
			} else {
				requestHere = newElevator.BackupInfo.FullHallRequests[floor][btn]
			}
			elevio.SetButtonLamp(elevio.ButtonType(btn), floor, requestHere)
		}
	}
	return newElevator
}

/*type Pair struct {
	First int
	Second int
}*/
