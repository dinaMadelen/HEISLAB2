package timer

import (
	"time"
)

var (
	timerEndTime float64
	timerActive  bool
	timeoutChan  chan bool
)

func TimerInit(tc chan bool) {
	timeoutChan = tc
}

// getWallTime returns the current wall time in seconds.
func TimerGetWallTime() float64 {
	return float64(time.Now().UnixNano()) / 1e9
}

// Start initializes the timer with a specified duration (in seconds).
func TimerStart(duration float64) {
	timerEndTime = TimerGetWallTime() + duration
	timerActive = true

	go func() {
		time.Sleep(time.Duration(duration * float64(time.Second)))
		if timerActive && TimerGetWallTime() >= timerEndTime {
			timeoutChan <- true // Send timeout event
		}
	}()
}

// Stop deactivates the timer.
func TimerStop() {
	timerActive = false
	select {
	case <-timeoutChan: // Clear the timeout signal if already triggered
	default:
	}
}

// TimedOut checks if the timer has timed out.
func TimerTimedOut() bool {
	return timerActive && TimerGetWallTime() > timerEndTime
}
