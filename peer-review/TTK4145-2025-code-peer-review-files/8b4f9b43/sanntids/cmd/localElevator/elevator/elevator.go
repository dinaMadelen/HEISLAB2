package elevator
import (
	"sanntids/cmd/localElevator/config"
	"Driver-go/elevio"
)


type ElevatorBehaviour int
const (
    EB_Idle ElevatorBehaviour = iota
    EB_DoorOpen
    EB_Moving
)

func Eb_toString(eb ElevatorBehaviour) string {
    switch eb {
    case EB_Idle:
        return "idle"
    case EB_DoorOpen:
        return "door open"
    case EB_Moving:
        return "moving"
    default:
        return "unknown"
    }
}

func Md_toString(md elevio.MotorDirection) string {
    switch md {
    case elevio.MD_Up:
        return "Up"
    case elevio.MD_Down:
        return "Down"
    case elevio.MD_Stop:
        return "stop"
    default:
        return "unknown"
    }
}

type Button int
const (
    BT_HallUp   Button = iota
    BT_HallDown
    BT_Cab
    N_BUTTONS // This will be 3
)

type Elevator struct {
    Floor     int
    MotorDirection      elevio.MotorDirection
    Requests  [config.N_FLOORS][config.N_BUTTONS]bool
    Behaviour ElevatorBehaviour

    Config struct {
        ClearRequestVariant config.ClearRequestVariant
        DoorOpenDuration_s  float64
    }
}


func ElevatorInit() Elevator {
    // Create a zero-initialized fixed-size 2D array for Requests
    var requests [config.N_FLOORS][config.N_BUTTONS]bool

    // Initialize elevator with default values
    e := Elevator{
        Floor:          0,
        MotorDirection: elevio.MD_Stop,  // Default motor direction
        Requests:       requests,
        Behaviour:      EB_Idle,         // Default behaviour
    }

    // Configure additional settings
    e.Config.ClearRequestVariant = config.CV_All               // Default request clearing variant
    e.Config.DoorOpenDuration_s  = config.DoorOpenDuration_s  

    return e
}