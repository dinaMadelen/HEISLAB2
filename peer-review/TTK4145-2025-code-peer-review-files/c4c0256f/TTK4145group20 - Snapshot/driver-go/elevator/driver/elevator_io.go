package driver

import (
	"Driver-go/elevator/types"
	"fmt"
	"net"
	"sync"
	"time"
)

// Constants
const _pollRate = 25 * time.Millisecond // Polling rate for checking inputs from the hardware

// Global variables
var _initialized bool = false      // Flag to check if the driver has been initialized
var _numFloors = types.N_FLOORS // Number of floors in the elevator
var _mtx sync.Mutex                // Mutex for thread safety in the driver functions
var _conn net.Conn                 // TCP connection to the elevator server

// Init initializes the driver by establishing a connection to the elevator hardware
// It takes an address for the server and the number of floors as parameters
func Init(addr string, numFloors int) {
	if _initialized {
		fmt.Println("Driver already initialized!")
		return
	}
	_numFloors = numFloors // Set number of floors in the elevator
	_mtx = sync.Mutex{}    // Initialize mutex for thread-safety
	var err error
	_conn, err = net.Dial("tcp", addr) // Establish connection to the elevator server
	if err != nil {
		panic(err.Error()) // Panic if there is an error in connection
	}
	_initialized = true // Mark the driver as initialized
}

// SetMotorDirection sends a command to set the motor direction (up, down, or stop)
func SetMotorDirection(dir types.MotorDirection) {
	write([4]byte{1, byte(dir), 0, 0}) // Send motor direction command to the elevator hardware
}

// SetButtonLamp controls the button lamps (lights that indicate button presses)
func SetButtonLamp(button types.ButtonType, floor int, value bool) {
	write([4]byte{2, byte(button), byte(floor), toByte(value)}) // Send button lamp state (on/off) to the hardware
}

// SetFloorIndicator sets the floor indicator to show the current floor
func SetFloorIndicator(floor int) {
	write([4]byte{3, byte(floor), 0, 0}) // Update the floor indicator
}

// SetDoorOpenLamp controls the door open lamp (light indicating whether the door is open)
func SetDoorOpenLamp(value bool) {
	write([4]byte{4, toByte(value), 0, 0}) // Set the door open lamp (on/off)
}

// SetStopLamp controls the stop lamp (indicates whether the stop button is pressed)
func SetStopLamp(value bool) {
	write([4]byte{5, toByte(value), 0, 0}) // Set the stop lamp (on/off)
}

// PollButtons checks for button presses and sends events to the receiver channel
func PollButtons(receiver chan<- types.ButtonEvent) {
	prev := make([][3]bool, _numFloors) // Array to store previous button states
	for {
		time.Sleep(_pollRate) // Poll every 25ms
		for f := 0; f < _numFloors; f++ {
			for b := types.ButtonType(0); b < 3; b++ {
				v := GetButton(b, f)               // Get current state of the button
				if v != prev[f][b] && v{ // If state changes and button is pressed
					receiver <- types.ButtonEvent{
						Floor: f, 
						Button: types.ButtonType(b),
					} // Send event to receiver channel
				}
				prev[f][b] = v // Update the previous state of the button
			}
		}
	}
}

// PollFloorSensor monitors the floor sensor and sends updates to the receiver channel
func PollFloorSensor(receiver chan<- int) {
	prev := -1 // Initialize the previous floor sensor value
	for {
		time.Sleep(_pollRate)     // Poll every 25ms
		v := GetFloor()           // Get the current floor
		if v != prev && v != -1 { // If the floor changes and is not invalid (-1)
			receiver <- v // Send the new floor to the receiver channel
		}
		prev = v // Update the previous floor sensor value
	}
}

// PollStopButton checks for the stop button press and sends events to the receiver channel
func PollStopButton(receiver chan<- bool) {
	prev := false // Initialize the previous state of the stop button
	for {
		time.Sleep(_pollRate) // Poll every 25ms
		v := GetStop()        // Get the state of the stop button
		if v != prev {        // If the state changes
			receiver <- v // Send the new stop button state to the receiver channel
		}
		prev = v // Update the previous state of the stop button
	}
}

// PollObstructionSwitch checks for an obstruction and sends events to the receiver channel
func PollObstructionSwitch(receiver chan<- bool) {
	prev := false // Initialize the previous state of the obstruction switch
	for {
		time.Sleep(_pollRate) // Poll every 25ms
		v := GetObstruction() // Get the current state of the obstruction switch
		if v != prev {        // If the state changes
			receiver <- v // Send the new obstruction state to the receiver channel
		}
		prev = v // Update the previous state of the obstruction switch
	}
}

// GetButton checks the current state of a specific button (floor and button type)
func GetButton(button types.ButtonType, floor int) bool {
	a := read([4]byte{6, byte(button), byte(floor), 0}) // Read the button state from the hardware
	return toBool(a[1])                                 // Convert the byte value to a boolean (pressed or not)
}

// GetFloor reads the current floor from the floor sensor
func GetFloor() int {
	a := read([4]byte{7, 0, 0, 0}) // Read the floor sensor state
	if a[1] != 0 {
		return int(a[2]) // Return the floor number if valid
	} else {
		return -1 // Return -1 if no valid floor is detected
	}
}

// GetStop checks the state of the stop button (pressed or not)
func GetStop() bool {
	a := read([4]byte{8, 0, 0, 0}) // Read the stop button state
	return toBool(a[1])            // Convert the byte value to a boolean (pressed or not)
}

// GetObstruction checks the state of the obstruction switch (blocked or not)
func GetObstruction() bool {
	a := read([4]byte{9, 0, 0, 0}) // Read the obstruction switch state
	return toBool(a[1])            // Convert the byte value to a boolean (blocked or not)
}

// read sends a request to the hardware and reads the response
func read(in [4]byte) [4]byte {
	_mtx.Lock()         // Lock mutex to ensure thread safety
	defer _mtx.Unlock() // Unlock mutex when the function exits

	_, err := _conn.Write(in[:]) // Send request to the hardware
	if err != nil {
		panic("Lost connection to Elevator Server") // Panic if there is an error in communication
	}

	var out [4]byte
	_, err = _conn.Read(out[:]) // Read the response from the hardware
	if err != nil {
		panic("Lost connection to Elevator Server") // Panic if there is an error in reading response
	}

	return out // Return the response data
}

// write sends a command to the elevator hardware
func write(in [4]byte) {
	_mtx.Lock()         // Lock mutex to ensure thread safety
	defer _mtx.Unlock() // Unlock mutex when the function exits

	_, err := _conn.Write(in[:]) // Send command to the hardware
	if err != nil {
		panic("Lost connection to Elevator Server") // Panic if there is an error in communication
	}
}

// toByte converts a boolean value to a byte (0 for false, 1 for true)
func toByte(a bool) byte {
	var b byte = 0
	if a {
		b = 1
	}
	return b
}

// toBool converts a byte value to a boolean (0 for false, non-zero for true)
func toBool(a byte) bool {
	var b bool = false
	if a != 0 {
		b = true
	}
	return b
}
