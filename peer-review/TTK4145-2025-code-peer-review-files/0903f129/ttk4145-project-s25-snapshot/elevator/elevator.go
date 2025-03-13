package elevator

const (
	N_FLOORS  = 4
	N_BUTTONS = 3
)

type ElevatorState int

const (
	Idle ElevatorState = iota
	Moving
	DoorOpen
)
type ElevatorDir int

const (
	Down ElevatorDir = -1
	Stop ElevatorDir = 0
	Up   ElevatorDir = 1
)

type Elevator struct {
	Floor 	int
	State 	ElevatorState
	Dir   	ElevatorDir
	Queue 	[N_FLOORS][N_BUTTONS]bool 
}

type ButtonType int
const (
	BT_HallUp   ButtonType = 0
	BT_HallDown	ButtonType = 1
	BT_Cab      ButtonType = 2
)
type Order struct {
	Floor 	int
	Button 	ButtonType
}

type OrderUpdate struct {
	Floor 	int
	Button 	ButtonType
	Served 	bool
}

