package elevator

import (
	"fmt"
	"main/elevio"
	"strings"
)

const (
	N_FLOORS  int = 4
	N_BUTTONS int = 3
)

type ElevatorBehaviour int
type ClearRequestVariant int
type Direction int
type Button int

type Elevator struct {
	Floor     int
	Dirn      elevio.MotorDirection
	Requests  [N_FLOORS][N_BUTTONS]bool
	Behaviour ElevatorBehaviour

	Config struct {
		ClearRequestVariant ClearRequestVariant
		DoorOpenDuration_s  float64
	}	
}

// Enum for clear request variant
const (
	CV_All ClearRequestVariant = iota
	CV_InDirn
)

// Enum for elevator behaviour
const (
	EB_Idle ElevatorBehaviour = iota
	EB_DoorOpen
	EB_Moving
)

// map elevator behaviour to strings for printing
var EB_to_string = map[ElevatorBehaviour]string{
	EB_Idle:     "EB_idle",
	EB_DoorOpen: "EB_doorOpen",
	EB_Moving:   "EB_moving",
}

// map elevator direction to strings
var Dirn_to_string = map[elevio.MotorDirection]string{
	elevio.MD_Up:   "MD_up",
	elevio.MD_Down: "MD_down",
	elevio.MD_Stop: "MD_stop",
}

// map buttons to strings
var Button_to_string = map[elevio.ButtonType]string{
	elevio.BT_HallUp:   "B_hallUp",
	elevio.BT_HallDown: "B_hallDown",
	elevio.BT_Cab:      "B_cab",
}

// map request variant to strings
var CV_to_string = map[ClearRequestVariant]string{
	CV_All:    "CV_all",
	CV_InDirn: "CV_inDirn",
}

// Returns an uninitialised elevator object to be used
// and configured in the main loop and fsm
func Elevator_uninitialised() Elevator {
	uninitialised_elevator := Elevator{
		Floor:     -1,
		Dirn:      elevio.MD_Stop,
		Behaviour: EB_Idle,
		Config: struct {
			ClearRequestVariant ClearRequestVariant
			DoorOpenDuration_s  float64
		}{
			ClearRequestVariant: CV_InDirn,
			DoorOpenDuration_s:  3.0,
		},
	}

	return uninitialised_elevator
}

// Prints all the stats of the elevator
func Elevator_print(elevator Elevator) {
    stats := fmt.Sprintf("Floor: %d\nDirection: %s\nBehaviour: %s\nClear request variant: %s\nDoor open duration: %.1f\n",
        elevator.Floor, Dirn_to_string[elevator.Dirn], EB_to_string[elevator.Behaviour], CV_to_string[elevator.Config.ClearRequestVariant], elevator.Config.DoorOpenDuration_s)

    requests := "Requests:\n"
    for floor := 0; floor < N_FLOORS; floor++ {
        requests += fmt.Sprintf("  Floor %d: [", floor)
        for btn := 0; btn < N_BUTTONS; btn++ {
            if elevator.Requests[floor][btn] {
                requests += fmt.Sprintf(" %s ", Button_to_string[elevio.ButtonType(btn)])
            } else {
                requests += " - "
            }
        }
        requests += "]\n"
    }

    statsLines := splitLines(stats)
    requestsLines := splitLines(requests)

    maxLines := max(len(statsLines), len(requestsLines))

    for i := 0; i < maxLines; i++ {
        if i < len(statsLines) {
            fmt.Printf("%-40s", statsLines[i])
        } else {
            fmt.Printf("%-40s", "")
        }
        if i < len(requestsLines) {
            fmt.Print(requestsLines[i])
        }
        fmt.Println()
    }
    fmt.Print("\n")
}

func splitLines(s string) []string {
	return strings.Split(strings.TrimRight(s, "\n"), "\n")
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
