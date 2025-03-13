package timer

import (
	"time"
)

var (
	timerEndTime float64
	timerActive  bool
)

// getWallTime returns the current time 
func getWallTime() float64 {
	now := time.Now()
	return float64(now.Unix()) + float64(now.Nanosecond())*1e-9
}

// timerStart starts a timer with the given duration
func TimerStart(duration float64) {
	timerEndTime = getWallTime() + duration
	timerActive = true
}

// timerStop stops the timer
func TimerStop() {
	timerActive = false
}

// timerTimedOut checks if the timer has timed out
func TimerTimedOut() bool {
	return timerActive && getWallTime() > timerEndTime
}
