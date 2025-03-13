package elevator

import (
	"Driver-go/elevio"
	"fmt"
	"time"
)

//tar en elevatorBehaviour-verdi som argument og returnerer
//en peker til en streng som representerer navnet på veriden

//eb_toString-funksjonen tar en ElevatorBehaviour-verdi som input
//den bruker operatoren "? :" til å sammenligne verdien mot kjente
//enum-verdier (EB_Idle, EB_DoorOpen, EB_Moving)

type ElevatorBehaviour int

const (
	EB_Idle ElevatorBehaviour = iota //iota brukes for å forenkle
	//opprettelsen av sekvensielle verdier(starter på null
	//og øker automatisk med 1 for hver ny konstant i blokken)
	EB_DoorOpen
	EB_Moving
)

type ElevatorRole int

const (
	Slave ElevatorRole = 0
	Primary = 1
	Backup = 2
)

var ElevatorBehaviourToString = map[ElevatorBehaviour]string{
	EB_Idle:     "idle",
	EB_DoorOpen: "doorOpen",
	EB_Moving:   "moving",
}

func Eb_toString(eb ElevatorBehaviour) string {
	if str, exists := ElevatorBehaviourToString[eb]; exists {
		return str
	}
	return "EB_UNDEFINED"
}

// type HRAElevState struct {
//     Behavior    *int        `json:"behaviour"` //Pass på å gjøre dette til string før state sendes
//     Floor       int         `json:"floor"` 
//     Direction   *int 	    `json:"direction"`
//     CabRequests []bool      `json:"cabRequests"`
// }

// type HRAInput struct {
//     HallRequests    [][2]bool                   `json:"hallRequests"`
//     States          map[string]HRAElevState     `json:"states"`
// }

type ElevStates struct {
    Behaviour    string      
    Floor       int          
    Direction   string      
    CabRequests []bool 
	ID 			int    //string? 
}
//placeholder

// elevator struct representerer statene til heisen
type Elevator struct {
	IP 		  int //er det bedre med tall (1, 2, 3) basert på rolle, som da må oppdateres underveis?
	Role 	  ElevatorRole
	Floor     int
	PrevFloor int
	Dirn      elevio.MotorDirection
	Behaviour ElevatorBehaviour
	HallRequests [N_FLOORS][N_BUTTONS-1]bool 
	CabRequests [N_FLOORS]bool
	AssignedRequests [N_FLOORS][N_BUTTONS-1]bool
	RequestsToDo  [N_FLOORS][N_BUTTONS]bool
	Config    ElevatorConfig
	State HRAElevState
}

const (
	N_FLOORS   = 4
	N_BUTTONS  = 3
	B_HallUp   = 0
	B_HallDown = 1
	B_Cab      = 2
)

// Nytt, usikkert
type ElevatorOrder struct {
	Order elevio.ButtonEvent
	ElevatorIP 	  int
}

//


func Elevator_print(e Elevator) {
	fmt.Println(" +-----------------+")
	fmt.Printf("|floor = %-2d          |\n", e.Floor)
	fmt.Printf("  |dirn  = %-12.12s|\n", e.Dirn)
	fmt.Printf("  |behav = %-12.12s|\n", e.Behaviour)
	fmt.Println(" +-----------------+")
	fmt.Println("  |  | up  | dn  | cab |")

	for f := N_FLOORS - 1; f >= 0; f-- {
		fmt.Printf("| %d", f)
		for btn := 0; btn < N_BUTTONS; btn++ {
			if (f == N_FLOORS-1 && btn == B_HallUp) || (f == 0 && btn == B_HallDown) {
				fmt.Print("|       ")
			} else {
				if e.RequestsToDo[f][btn] {
					fmt.Print("|   #   ")
				} else {
					fmt.Print("|   -   ")
				}
			}
		}
		fmt.Println("|")
	}
	fmt.Println(" +-----------------------+")
}

// definierer konfigurasjonsstruktur
type ElevatorConfig struct {
	ClearRequestVariant string
	DoorOpenDuration    time.Duration
}

// funksjonen for å returnere en uinitialisert heis
func Elevator_uninitialized() Elevator {
	return Elevator{
		IP: 000,
		Role: 0, // Defaul role: slave
		Floor:     -1,             //ugyldig etasje
		Dirn:      elevio.MD_Stop, //heisen er stoppet
		Behaviour: EB_Idle,        //inaktiv tilstand
		HallRequests: [N_FLOORS][N_BUTTONS-1]bool{{false, false}, {false, false}, {false, false}, {false, false}},
		CabRequests: [N_FLOORS]bool{false, false, false, false},
		AssignedRequests: [N_FLOORS][N_BUTTONS-1]bool{{false, false}, {false, false}, {false, false}, {false, false}},
		RequestsToDo: [N_FLOORS][N_BUTTONS]bool{{false, false, false}, {false, false, false}, {false, false, false}, {false, false, false}},
		Config: ElevatorConfig{
			ClearRequestVariant: "CV_InDirn",       //fjerner alle forespørsler
			DoorOpenDuration:    3.0 * time.Second, //3 sekunder døråpning
		},
		State: {
			HRAElevState.Behavior := &Be
		},
	}
}
