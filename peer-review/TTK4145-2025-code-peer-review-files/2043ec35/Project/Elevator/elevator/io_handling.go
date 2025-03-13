package elevator

import (
	"elevproj/Elevator/elevio"
	"elevproj/config"
	"time"
)

func ObstructionActivated(elev Elevator, b bool, timerStart_chan chan time.Duration) ElevatorBehaviour {
	var behaviour = elev.Behaviour
	if b {
		if behaviour == EB_dooropen {
			timerStart_chan <- time.Duration(config.MaxDuration)
			return EB_obstruct
			//elev.Behaviour = EB_obstruct
		} else {
			return behaviour
		}
	} else {
		timerStart_chan <- elev.DoorOpenDuration
		return EB_dooropen
		//Behaviour = EB_dooropen
	}
}

func StopActivated(elev Elevator, b bool, timerStart_chan chan time.Duration) (ElevatorBehaviour, bool) {
	var behaviour ElevatorBehaviour
	var stopPressed bool
	if b {
		elevio.SetMotorDirection(0)
		behaviour = EB_stop
		stopPressed = true

	} else {
		behaviour = EB_idle
		stopPressed = false

	}
	return behaviour, stopPressed
}
