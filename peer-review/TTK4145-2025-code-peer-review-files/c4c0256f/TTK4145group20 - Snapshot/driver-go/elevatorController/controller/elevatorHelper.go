package controller

import (
	"Driver-go/elevator/driver"
	. "Driver-go/elevator/types"
	"Driver-go/elevatorController/timer"
	"time"
)

var sharedInfo ElevatorSharedInfo

// GetElevatorInfo returns a copy of the shared elevator state.
func GetElevatorInfo() ElevatorInfo {
	sharedInfo.Mutex.RLock()
	defer sharedInfo.Mutex.RUnlock()

	return ElevatorInfo{
		Available: sharedInfo.Available,
		Behaviour: sharedInfo.Behaviour,
		Direction: sharedInfo.Direction,
		Floor:     sharedInfo.Floor,
	}
}

// updateElevatorInfo updates the shared elevator state.
func updateElevatorInfo(e Elevator) {
	sharedInfo.Mutex.Lock()
	defer sharedInfo.Mutex.Unlock()

	sharedInfo.Behaviour = e.Behaviour
	sharedInfo.Direction = e.Direction
	sharedInfo.Floor = e.Floor
}

// setElevatorAvailability sets the elevator's availability flag.
func setElevatorAvailability(value bool) {
	sharedInfo.Mutex.Lock()
	defer sharedInfo.Mutex.Unlock()

	sharedInfo.Available = value
}

// elevatorInit initializes the elevator hardware and returns the initial state.
func elevatorInit(floorSensorCh <-chan int) Elevator {
	driver.SetDoorOpenLamp(false)

	for f := 0; f < N_FLOORS; f++ {
		for b := ButtonType(0); b < N_BUTTONS; b++ {
			driver.SetButtonLamp(b, f, false)
		}
	}

	driver.SetMotorDirection(MD_Down)
	currentFloor := <-floorSensorCh
	driver.SetMotorDirection(MD_Stop)
	driver.SetFloorIndicator(currentFloor)

	return Elevator{
		Floor:     currentFloor,
		Direction: ED_Stop,
		Requests:  [N_FLOORS][N_BUTTONS]bool{},
		Behaviour: EB_Idle,
	}
}

// startTimerChannel starts the custom timer for the given duration (in seconds)
// and returns a channel that will signal when the timer expires.
func startTimerChannel(t *timer.Timer, duration int) <-chan bool {
	ch := make(chan bool, 1)
	t.Start(float64(duration))
	go func() {
		for {
			if t.TimedOut() {
				ch <- true
				return
			}
			time.Sleep(50 * time.Millisecond)
		}
	}()
	return ch
}

// timerStop stops the custom timer.
func timerStop(t *timer.Timer) {
	t.Stop()
}

// directionConverter converts the elevator's Direction_t to the motor's MotorDirection_t.
func directionConverter(dir ElevDirection) MotorDirection {
	switch dir {
	case ED_Up:
		return MD_Up
	case ED_Down:
		return MD_Down
	case ED_Stop:
		return MD_Stop
	}
	return MD_Stop
}
