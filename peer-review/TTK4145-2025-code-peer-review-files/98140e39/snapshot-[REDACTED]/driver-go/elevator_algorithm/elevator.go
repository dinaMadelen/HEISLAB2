package elevatoralgorithm

import (
	"fmt"
	"log"
	"os"
	"path"
	"time"

	"gopkg.in/yaml.v3"
)

func loadConfig() (config, error) {
	c := config{}
	file, err := os.Open(configPath)
	if err != nil {
		fmt.Println("Error reading file")
		return c, err
	}
	defer file.Close()

	err = yaml.NewDecoder(file).Decode(&c)
	if err != nil {
		fmt.Println("Error decoding file")
		return c, err
	}
	return c, nil
}

const (
	NumFloors  = 4
	NumButtons = 3
)

var configPath = path.Join("driver-go", "elevator_algorithm", "elevator_config.yaml")

type elevatorBehaviour int

const (
	idle elevatorBehaviour = iota
	doorOpen
	moving
)

type clearRequestVariant int

const (
	clearAll clearRequestVariant = iota
	clearSameDir
)

type direction int

const (
	down direction = iota - 1
	stop
	up
)

type Button int

const (
	hallUp Button = iota
	hallDown
	cabButton
)

type config struct {
	ClearRequestVariant clearRequestVariant `yaml:"ClearRequestVariant"`
	DoorOpenDuration    time.Duration       `yaml:"DoorOpenDuration"`
}

type Elevator struct {
	Floor     int
	direction direction
	requests  [NumFloors][NumButtons]bool
	behaviour elevatorBehaviour
	config    config
}

type behaviourPair struct {
	dir       direction
	behaviour elevatorBehaviour
}

func dirToString(direction direction) string {
	switch direction {
	case up:
		return "Direction_up"
	case stop:
		return "Direction_down"
	case down:
		return "Direction_down"
	default:
		return "Undefined direction"
	}
}

func buttonToString(button Button) string {
	switch button {
	case hallUp:
		return "hall_button_up"
	case hallDown:
		return "hall_button_down"
	case cabButton:
		return "cab_button"
	default:
		return "Undefined button"
	}
}

func behaviourToString(behaviour elevatorBehaviour) string {
	switch behaviour {
	case idle:
		return "idle"
	case doorOpen:
		return "door is open"
	case moving:
		return "moving"
	default:
		return "undefined behaviour"
	}
}

func (e *Elevator) PrintElevator() {
	fmt.Println("  +--------------------+")
	fmt.Printf("  |floor = %-2d          |\n", e.Floor)
	fmt.Printf("  |dirn  = %-12.12s|\n", dirToString(e.direction))
	fmt.Printf("  |behav = %-12.12s|\n", behaviourToString(e.behaviour))

	fmt.Println("  +--------------------+")
	fmt.Println("  |  | up  | dn  | cab |")
	for f := NumFloors - 1; f >= 0; f-- {
		fmt.Printf("  | %d", f)
		for btn := 0; btn < NumButtons; btn++ {
			if (f == NumFloors-1 && btn == int(hallUp)) || (f == 0 && btn == int(hallDown)) {
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

func MakeUninitializedelevator() Elevator {
	config, err := loadConfig()
	if err != nil {
		// TODO: Retry here instead of just crashing
		// This is not very fault tolerant
		log.Fatal("Failed to initialize elevator from .yaml file")
	}

	return Elevator{
		Floor:     -1,
		direction: stop,
		behaviour: idle,
		config:    config,
	}
}

func CreateMockElevator() Elevator {
	config, err := loadConfig()
	if err != nil {
		// TODO: Retry here instead of just crashing
		// This is not very fault tolerant
		log.Fatal("Failed to initialize elevator from .yaml file")
	}

	return Elevator{
		Floor:     3,
		direction: stop,
		behaviour: idle,
		requests:  [NumFloors][NumButtons]bool{{false, false, false}, {false, false, false}, {false, false, false}, {false, false, false}},
		config:    config,
	}
}

func (e *Elevator) GetFloor() int {
	return e.Floor
}
