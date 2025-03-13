package timer

import (
	"time"
)

var (
	timerEndTime time.Time
	timerActive  bool
)

func getWallTime() time.Time {
	return time.Now()
}

func TimerStart(duration float64) {
	timerEndTime = getWallTime().Add(time.Duration(duration * float64(time.Second)))
	timerActive = true
}

func TimerStop() {
	timerActive = false
}

func TimerTimedOut() bool {
	return timerActive && getWallTime().After(timerEndTime)
}

// Used for implementing the obstruction button
func AddTime(duration float64) {
	timerEndTime = getWallTime().Add(time.Duration(duration * float64(time.Second)))
}

func PollTimer(receiver chan<- bool) {
	for {
		time.Sleep(10 * time.Millisecond) // Adjust the polling rate as needed
		if TimerTimedOut() {
			receiver <- true
			TimerStop() // Stop the timer after timeout
		}
	}
}
