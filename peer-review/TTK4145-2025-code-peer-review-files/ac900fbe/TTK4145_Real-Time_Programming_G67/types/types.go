package types


const (
	NUMFLOORS      = 4
	NUMBUTTONTYPE  = 3
	NUMHALLBUTTONS = 2
)

// ElevatorBehaviour is an enum type for the different states an elevator can be in.
type ElevatorBehaviour int

const (
	EB_Idle ElevatorBehaviour = iota
	EB_DoorOpen
	EB_Moving
)

// ClearRequestVariant is an enum type for the different ways to clear requests.
type ClearRequestVariant int

const (
	CV_All ClearRequestVariant = iota
	CV_InDirn
)

type MotorDirection int

const (
	MD_Up   MotorDirection = 1
	MD_Down MotorDirection = -1
	MD_Stop MotorDirection = 0
)

type ButtonType int

const (
	BT_HallUp   ButtonType = 0
	BT_HallDown ButtonType = 1
	BT_Cab      ButtonType = 2
)

// ButtonEvent is a struct type for the events that can be triggered by pressing a button, containing the floor and the button type.
type ButtonEvent struct {
	Floor  int
	Button ButtonType
}

// OrderEvent is a struct type for the events that can be triggered by an elevator completing an order, 
// containing the elevator ID, whether the order is completed, and the orders.
type OrderEvent struct {
	ElevatorID string
	Completed  bool
	Orders     []ButtonEvent
}

// DirnBehaviourPair is a struct type for the pair of motor direction and elevator behaviour.
type DirnBehaviourPair struct {
	Dirn      MotorDirection
	Behaviour ElevatorBehaviour
}

// ClearRequestCallback is a function type for clearing requests.
type ClearRequestCallback func(button ButtonType, floor int)

// OrderMatrix is a matrix type for the orders in the elevator system.
type OrderMatrix [NUMFLOORS][NUMBUTTONTYPE]bool

// GlobalOrderMap is a map type for all the orders in the elevator system, and the elevator ID as the key.
type GlobalOrderMap map[string]OrderMatrix

// Elevator is a struct type for the elevator, containing the ID, floor, motor direction, 
// requests matrix, behaviour, availability, and configuration.
type Elevator struct {
	ID        string
	Floor     int
	Dirn      MotorDirection
	Requests  OrderMatrix
	Behaviour ElevatorBehaviour
	Avaliable bool
	Config    struct {
		ClearRequestVariant ClearRequestVariant
		DoorOpenDuration    float64
		TimeBetweenFloors   float64
	}
}

// NetworkMessage is a struct type for the messages sent between the master and the slaves.
type NetworkMessage struct {
	MsgType    string
	MsgData    interface{}
	Receipient Receipient
}

// Receipient is an enum type for the different types of receipients for the network messages.
type Receipient int

const (
	All Receipient = iota
	Master
)