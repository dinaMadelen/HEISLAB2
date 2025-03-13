package elevator

import (
	"fmt"
	"time"
)

type ElevatorBehaviour int

const (
	BEHAVIOUR_IDLE = iota
	BEHAVIOUR_DOOR_OPEN
	BEHAVIOUR_MOVING
)

type ClearRequestVariant int

const (
	CV_All = iota

	CV_InDirn
)

type Elevator struct {
	floor     int
	direction Dir
	requests  [NUM_FLOORS][NUM_BUTTONS]bool
	behaviour ElevatorBehaviour
	config    config
}

type config struct {
	clearRequestVariation ClearRequestVariant
	doorOpenDuration      time.Duration
}

type DirBehaviourPair struct {
	dir Dir
	//direction of the elevator: DIR_UP, DIR_DOWN, DIR_STOP

	behaviour ElevatorBehaviour
	//states of the elevator: BEHAVIOUR_IDLE, BEHAVIOUR_DOOR_OPEN, BEHAVIOUR_MOVING
}

const (
	NUM_FLOORS  = 4
	NUM_BUTTONS = 3
)

type Dir int

const (
	DIR_DOWN Dir = iota - 1
	DIR_STOP
	DIR_UP
)

type Button int

const (
	BTN_HALLUP Button = iota
	BTN_HALLDOWN
	BTN_HALLCAB
)


func ElevioDirToString(d Dir) string {
	switch d {
	case DIR_UP:
		return "up"
	case DIR_DOWN:
		return "down"
	case DIR_STOP:
		return "stop"
	default:
		return "udefined"
	}
}

func ElevioButtonToString(b Button) string {
	switch b {
	case BTN_HALLUP:
		return "HallUp"
	case BTN_HALLDOWN:
		return "HallDown"
	case BTN_HALLCAB:
		return "Cab"
	default:
		return "undefined"
	}
}

func EbToString(behaviour ElevatorBehaviour) string {
	switch behaviour {
	case BEHAVIOUR_IDLE:
		return "idle"
	case BEHAVIOUR_DOOR_OPEN:
		return "doorOpen"
	case BEHAVIOUR_MOVING:
		return "moving"
	default:
		return "undefined"
	}
}

//this function just prints the current elevator status in the terminal
//If the code works properly at some point, any changes in the terminal that the simulator is run in
//should be visible in the terminal that the go-program is run in as well ;)

func (e *Elevator) print() {
	fmt.Println("  +--------------------+")
	fmt.Printf("  |floor = %-2d          |\n", e.floor)
	fmt.Printf("  |dirn  = %-12.12s|\n", ElevioDirToString(e.direction))
	fmt.Printf("  |behav = %-12.12s|\n", EbToString(e.behaviour))

	fmt.Println("  +--------------------+")
	fmt.Println("  |  | up  | dn  | cab |")
	for f := NUM_FLOORS - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := 0; btn < NUM_BUTTONS; btn++ {
			if (f == NUM_FLOORS-1 && btn == int(BTN_HALLUP)) || (f == 0 && btn == int(BTN_HALLDOWN)) {
				fmt.Print("|     ")
			} else {
				if e.requests[f][btn] {
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

//Defalult elevator that starts in floor: -1, this doesnt make sense, but it does
//We cant initialize the elevator in a spesific floor, and PollFloorSensor() will update the variable to the correct
//floor as soon as the elevator starts moving i think

func MakeUninitializedelevator() Elevator {
	return Elevator{
		floor:     -1,
		direction: DIR_STOP,
		behaviour: BEHAVIOUR_IDLE,
		config: config{
			clearRequestVariation: CV_InDirn,
			doorOpenDuration:      3.0,
		},
	}
}





// func send_requests(e *Elevator) {
// 	conn, err := net.Dial("tcp", "10.100.23.33:8080")
// 	if err != nil {
// 		fmt.Println("Error connecting to server:", err)
// 	}
// 	defer conn.Close()

// 	str := "requests:"
// 	for i := 0; i < 4; i++ {
// 		for j := 0; j < 3; j++ {
// 			str += "_" + fmt.Sprint(e.requests[i][j])
// 		}
// 	}

// 	_, err = conn.Write([]byte(str))
// 	if err != nil {
// 		fmt.Println("Error sending message:", err)
// 		return
// 	}
// 	time.Sleep(time.Second)

// }
