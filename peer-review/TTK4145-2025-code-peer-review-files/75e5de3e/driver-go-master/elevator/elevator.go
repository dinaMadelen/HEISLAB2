package elevator

import "fmt"

// Antall etasjer og knapper i heissystemet
const (
	NFloors = 4
	NBtns   = 3
)

// Definerer retninger for heisen
type Dirn int

const (
	Down Dirn = -1
	Stop Dirn = 0
	Up   Dirn = 1
)

// Definerer knappetyper
type Button int

const (
	HallUp Button = iota
	HallDown
	Cab
)

// Konverterer heisens retning til en string for utskrift
func ElevatorDirnToString(d Dirn) string {
	switch d {
	case Up:
		return "up"
	case Down:
		return "down"
	case Stop:
		return "stop"
	default:
		return "undefined"
	}
}

// Heisens mulige tilstander
type ElevatorBehaviour int

const (
	Idle ElevatorBehaviour = iota
	DoorOpen
	Moving
)

// Representerer en heis med tilstand og konfigurasjon
type Elevator struct {
	Floor     int
	Dirn      Dirn
	Requests  [NFloors][NBtns]int
	Behaviour ElevatorBehaviour
	Config    struct {
		ClearRequestVariant int
		DoorOpenDuration_s  float64
	}
}

// Konverter heisens state til en string for utskrift
func (eb ElevatorBehaviour) ElevatorBehaviorToString() string {
	var return_string string
	switch eb {
	case Idle:
		return_string = "idle"
	case DoorOpen:
		return_string = "doorOpen"
	case Moving:
		return_string = "moving"
	}
	return return_string
}

// Terminalpynt
func ElevatorPrint(e Elevator) {
	fmt.Printf("  +--------------------+\n")
	fmt.Printf(
		"  |floor = %-2d          |\n"+
			"  |dirn  = %-12.12s|\n"+
			"  |behav = %-12.12s|\n",
		e.Floor,

		ElevatorDirnToString(e.Dirn),
		e.Behaviour.ElevatorBehaviorToString(),
	)
	fmt.Printf("  +--------------------+\n")
	fmt.Printf("  |  | up  | dn  | cab |\n")
	for f := NFloors - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := 0; btn < NBtns; btn++ {
			if (f == NFloors-1 && btn == int(HallUp)) ||
				(f == 0 && btn == int(HallDown)) {
				fmt.Printf("|     ")
			} else {
				if e.Requests[f][btn] == 1 {
					fmt.Printf("|  #  ")
				} else {
					fmt.Printf("|  -  ")
				}
			}
		}
		fmt.Printf("|\n")
	}
	fmt.Printf("  +--------------------+\n")
}

// Oppretter en uinitialisert heis med standardinstillinger
func ElevatorUninitialized() Elevator {
	return Elevator{
		Floor:     -1,
		Dirn:      0,
		Behaviour: Idle,
		Requests:  [NFloors][NBtns]int{},
		Config: struct {
			ClearRequestVariant int
			DoorOpenDuration_s  float64
		}{ClearRequestVariant: 0, DoorOpenDuration_s: 3.0},
	}
}
