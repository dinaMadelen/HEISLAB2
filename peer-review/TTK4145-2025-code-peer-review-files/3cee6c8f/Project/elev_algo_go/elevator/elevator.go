package elevator

import (
	"Driver-go/elevio"
	"bufio"
	"fmt"
	"os"
	"strings"
)

const BroadcastingIP = "255.255.255.255"

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

const (
	NumFloors  = 4
	NumButtons = 3
)

type Elevator struct {
	Config struct {
		DoorOpenDuration_s  float64
		ClearRequestVariant string
	}
	Dirn      int
	Behaviour int
	Requests  [][]bool
	Floor     int
}

type ElevState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type ElevWorldView struct {
	SenderElevState ElevState
	HallRequests    [4][2]bool
	ID              string
}

const (
	EB_Idle = iota
	EB_DoorOpen
	EB_Moving
)

const (
	D_Down = iota - 1 // -1
	D_Stop            // 0
	D_Up              // 1
)

func (e *Elevator) Uninitialized() Elevator {
	elevator := Elevator{
		Requests: make([][]bool, NumFloors),
	}
	for i := range elevator.Requests {
		elevator.Requests[i] = make([]bool, NumButtons)
	}
	return elevator
}

func (e *Elevator) Initialize() {
	*e = e.Uninitialized()

	e.Dirn = D_Stop
	e.Behaviour = EB_Idle
	e.Floor = -1

	file, err := os.Open("config/elevator.con")
	if err != nil {
		fmt.Println("Error opening config file:", err)
		return
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "--clearRequestVariant") {
			fmt.Sscanf(line, "--clearRequestVariant %s", &e.Config.ClearRequestVariant)
		} else if strings.HasPrefix(line, "--doorOpenDuration_s") {
			fmt.Sscanf(line, "--doorOpenDuration_s %f", &e.Config.DoorOpenDuration_s)
		}
	}

	if err := scanner.Err(); err != nil {
		fmt.Println("Error reading config file:", err)
		return
	}

	fmt.Println("Config values read and assigned:")
	fmt.Println("clearRequestVariant:", e.Config.ClearRequestVariant)
	fmt.Println("doorOpenDuration_s:", e.Config.DoorOpenDuration_s)
}

func (e *Elevator) Elevator_print() {
	fmt.Println("  +--------------------+")
	fmt.Printf(
		"  |floor = %-2d          |\n"+
			"  |dirn  = %-12.12s|\n"+
			"  |behav = %-12.12s|\n",
		e.Floor,
		DirnToString(e.Dirn),
		EBToString(e.Behaviour),
	)
	fmt.Println("  +--------------------+")
	fmt.Println("  |  | up  | dn  | cab |")
	for f := len(e.Requests) - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := 0; btn < len(e.Requests[f]); btn++ {
			if (f == len(e.Requests)-1 && elevio.ButtonType(btn) == elevio.BT_HallUp) ||
				(f == 0 && elevio.ButtonType(btn) == elevio.BT_HallDown) {
				fmt.Print("|     ")
			} else {
				if e.Requests[f][btn] {
					fmt.Print("|  #  ")
				} else {
					fmt.Print("|  -  ")
				}
			}
		}
		fmt.Println("|")
	}
	fmt.Println("  +--------------------+")
}

func EBToString(eb int) string {
	switch eb {
	case EB_Idle:
		return "EB_Idle"
	case EB_DoorOpen:
		return "EB_DoorOpen"
	case EB_Moving:
		return "EB_Moving"
	default:
		return "EB_UNDEFINED"
	}
}

func DirnToString(dirn int) string {
	switch dirn {
	case D_Up:
		return "D_Up"
	case D_Down:
		return "D_Down"
	case D_Stop:
		return "D_Stop"
	default:
		return "D_UNDEFINED"
	}
}

func UpdateElevWorldView(e Elevator, localIP string) ElevWorldView {
	var worldview ElevWorldView

	for f := e.Floor + 1; f < NumFloors; f++ {
		for btn := 0; btn < NumButtons; btn++ {
			if btn == 0 {
				worldview.HallRequests[f][btn] = e.Requests[f][btn]

			} else if btn == 1 {
				worldview.HallRequests[f][btn] = e.Requests[f][btn]

			} else if btn == 2 {
				worldview.SenderElevState.CabRequests[f] = e.Requests[f][btn]
			}

		}
	}

	worldview.SenderElevState.Behavior = EBToString(e.Behaviour)

	worldview.SenderElevState.Direction = DirnToString(e.Dirn)

	worldview.SenderElevState.Floor = e.Floor

	worldview.ID = localIP

	return worldview
}
