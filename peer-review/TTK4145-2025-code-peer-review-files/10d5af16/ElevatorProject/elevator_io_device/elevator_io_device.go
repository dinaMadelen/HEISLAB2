package elevator_io_device

import (
	"Driver-go/elevio"
	"fmt"
)

// definierer enum for Button og Dirn
type Button elevio.ButtonType
type Dirn int

const (
	//knappetyper, overflødig?
	B_HallUp   Button = 0
	B_HallDown Button = 1
	B_Cab      Button = 2

	//retningstyper starter på ny iota-sekvens
	D_Up   Dirn = 1
	D_Down Dirn = -1
	D_Stop Dirn = 0
)

// funksjonen som stimulerer initialisering av heismaskinvare
func Init() {
	Elevator_hardware_init()
}

// simulerer initialisering av heisens maskinvare
func Elevator_hardware_init() {
	fmt.Println("Initialising elevator hardware...")
	//her kan du vi legge til kode for maskinvareinitialisering
}

// funksjonen som simulerer å hente signalet fra en knapp på en spesifikk etasje
func Wrap_request_button(f int, b Button) int {
	return Elevator_hardware_get_button_signal(b, f)
}

// simulerer å hente knappesignal for en spesifikk etasje og knapp
func Elevator_hardware_get_button_signal(b Button, f int) int {
	fmt.Printf("Getting button signal for floor %d and button %d\n", f, b)
	return 1 //simulerer at signalet er aktivert
}

// funksjonen for å sette en lampe på en knapp (tillater å sette den på eller av)
func Wrap_request_button_light(f int, b elevio.ButtonType, v bool) {
	Elevator_hardware_set_button_lamp(b, f, v)
}

// simulerer å sette lampen til en spesifikk verdi for en knapp på en etasje
func Elevator_hardware_set_button_lamp(b elevio.ButtonType, f int, v bool) {
	//fmt.Printf("Setting button light for floor %d, button %d, value %d\n", f, b, v)
	//her kan vi legge til kode som setter lampeverdien på en knapp
	elevio.SetButtonLamp(b, f, v)
}

// funksjonen som simulerer å sette motorretning
func Wrap_motor_direction(d elevio.MotorDirection) {
	Elevator_hardware_set_motor_direction(d)
}

// simulerer å sette motorretningen til heisen
func Elevator_hardware_set_motor_direction(d elevio.MotorDirection) {
	fmt.Printf("Setting motor direction to %d\n", d)
	//her kan vi legge til kode som setter motorens retning
	elevio.SetMotorDirection(d)
}

// definerer en sturktur for å holde funksjoner relatert til heisinndata
type ElevInputDevice struct {
	FloorSensor   func() int
	RequestButton func(floor int, button Button) int
	Obstruction   func() bool
}

// simulerte funksjoner for å etterligne maskinvarefunksjonene
func Elevator_hardware_get_floor_sensor_signal() int {
	//simulerer deteksjon av etasjesensor
	return 1 //eks. heisen er i etasje 1
}

func Elevator_hardware_get_obstruction_signal() bool {
	//simulerer hindringssensor
	return false //eks. ingen hindring oppdaget
}

// funksjon for å returnere en instans av ElevInputDevice
func Elevio_getInputDevice() ElevInputDevice {
	return ElevInputDevice{
		FloorSensor:   Elevator_hardware_get_floor_sensor_signal,
		RequestButton: Wrap_request_button,
		Obstruction:   Elevator_hardware_get_obstruction_signal,
	}
}

// elevOutputDevice stuct holds function pointers for controlling elevator output devices
type ElevOutputDevice struct {
	FloorIndicator     func(floor int)
	RequestButtonLight func(floor int, button elevio.ButtonType, value bool)
	DoorLight          func(value int)
	StopButtonLight    func(value int)
	MotorDirection     func(direction elevio.MotorDirection)
}

// simulated hardware functios to mimic the actual hardware control
func Elevator_hardware_set_floor_indicator(floor int) {
	//simulate setting the floor indicator lamp
	println("Floor indicator set to:", floor)
}

func Elevator_hardware_set_door_open_lamp(value int) {
	//simulate setting the door open light
	print("Door light set to:", value)
}

func Elevator_hardware_set_stop_lamp(value int) {
	//simulate setting the stop lamp
	println("Stop button light set to:", value)
}

// function to return an instance of elevOutputDevice with function assignments
func Elevio_getOutputDevice() ElevOutputDevice {
	return ElevOutputDevice{
		FloorIndicator:     Elevator_hardware_set_floor_indicator,
		RequestButtonLight: Wrap_request_button_light,
		DoorLight:          Elevator_hardware_set_door_open_lamp,
		StopButtonLight:    Elevator_hardware_set_stop_lamp,
		MotorDirection:     Wrap_motor_direction,
	}
}

// mapper knapper og retninger til strenger
var ButtonToString = map[Button]string{
	Button(elevio.BT_HallUp):   "B_HallUp",
	Button(elevio.BT_HallDown): "B_HallDown",
	Button(elevio.BT_Cab):      "B_Cab",
}

var DirnToString = map[Dirn]string{
	D_Up:   "D_Up",
	D_Down: "D_Down",
	D_Stop: "D_Stop",
}

var StringToDirn = map[string]Dirn{
	"D_Up":   D_Up,
	"D_Down": D_Down,
	"D_Stop": D_Stop,
}

// funksjoner for å hente tilsvarende string
func Elevio_button_toString(b Button) string {
	if str, exists := ButtonToString[b]; exists {
		return str
	}
	return "D_UNDEFINED"
}

func Elevio_drin_toString(d Dirn) string {
	if str, exists := DirnToString[d]; exists {
		return str
	}
	return "D_UNDEFINED"
}