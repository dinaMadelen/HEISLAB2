package fsm

import (
	"fmt"
	"time"
)

// If this is changed, remember to change this in:IO.GO too
const _pollRate = 20 * time.Millisecond

// Function that polls the timer at a given rate. Sends a signal when the timer has timed out
func PollTimerTimeout(receiver chan<- bool) {
	for {
		time.Sleep(_pollRate) // Poll rate, adjust as needed
		if TimerTimedOut() {
			receiver <- true // Send a signal when the timer has timed out
		}
	}
}

var timerEndTime time.Time
var timerActive bool

func getWallTime() time.Time {
	return time.Now()
}

// starts a timer with a given duration
func TimerStart(duration float64) {
	fmt.Println("Timer started, for:", duration)
	timerEndTime = getWallTime().Add(time.Duration(duration * float64(time.Second)))
	timerActive = true
}

func TimerStop() {
	timerActive = false
}

func TimerTimedOut() bool {
	return timerActive && getWallTime().After(timerEndTime) && !(elevator.Obstruction)
}
