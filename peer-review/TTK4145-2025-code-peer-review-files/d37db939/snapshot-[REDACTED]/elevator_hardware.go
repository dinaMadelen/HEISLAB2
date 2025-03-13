package main

import (
	"log"
	"net"
	"sync"
)

var (
	sock      net.Conn
	sockMutex sync.Mutex
)

// MotorDirection represents the direction of elevator movement
type MotorDirection int
const (
	DIRN_DOWN MotorDirection = -1
	DIRN_STOP MotorDirection = 0
	DIRN_UP   MotorDirection = 1
)

// ButtonType represents the types of buttons in the elevator system (in the hall and in the cab)
type ButtonType int
const (
	BUTTON_HALL_UP ButtonType = iota
	BUTTON_HALL_DOWN
	BUTTON_CAB
)

// LampType represents the types of lamps on the elevator remote
type LampType int
const (
	LAMP_BUTTON LampType = iota
	LAMP_FLOOR_INDICATOR
	LAMP_DOOR
	LAMP_STOP
)

// initializing the connection to the elevator hardware
func hardwareInit() {
	var err error
	sock, err = net.Dial("tcp", "localhost:15657")
	if err != nil {
		// panic("Unable to connect to simulator server")
		log.Fatalf("Unable to connect to simulator server: %v", err)
	}
	sock.Write([]byte{0, 0, 0, 0})
}

// generic function for sending a command to the elevator and returning the response byte
func hardwareExecuteCommand(command byte, arg1 byte, arg2 byte, arg3 byte) byte {
	sockMutex.Lock()
	defer sockMutex.Unlock()

	sock.Write([]byte{command, arg1, arg2, arg3})
	buf := make([]byte, 4)
	sock.Read(buf)

	return buf[1]
}

// setting the elevator motor direction
func hardwareSetMotorDirection(dirn MotorDirection) {
	hardwareExecuteCommand(1, byte(dirn), 0, 0)
}

// retrieving the state of a specified button on a given floor
func hardwareGetButtonSignal(floor int, button ButtonType) int {
	return int(hardwareExecuteCommand(6, byte(button), byte(floor), 0))
}

// returning the current floor sensor signal
func hardwareGetFloorSensorSignal() int {
	result := hardwareExecuteCommand(7, 0, 0, 0)
	if result != 0 {
		return int(hardwareExecuteCommand(7, 0, 0, 0))
	}
	return -1
}

// returning the state of the stop button (1 if pressed, 0 if not)
func hardwareGetStopSignal() int {
	return int(hardwareExecuteCommand(8, 0, 0, 0))
}

// returning the state of the obstruction signal (1 if obstruction detected, 0 if not)
func hardwareGetObstructionSignal() int {
	return int(hardwareExecuteCommand(9, 0, 0, 0))
}

// toggling the various lamps
func hardwareLampToggle(lampType LampType, button ButtonType, floor int, lightOn int) {
	sockMutex.Lock()
	defer sockMutex.Unlock()

	var command []byte

	switch lampType {
	case LAMP_BUTTON:
		command = []byte{2, byte(button), byte(floor), byte(lightOn)}
	case LAMP_FLOOR_INDICATOR:
		command = []byte{3, byte(floor), 0, 0}
	case LAMP_DOOR:
		command = []byte{4, byte(lightOn), 0, 0}
	case LAMP_STOP:
		command = []byte{5, byte(lightOn), 0, 0}
	default:
		return
	}

	sock.Write(command)
}