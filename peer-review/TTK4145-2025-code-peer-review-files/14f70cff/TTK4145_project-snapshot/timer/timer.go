package timer

import (
	"time"
)

var timerEndTime time.Time
var timerActive bool

func TimerStart() {
	timerEndTime = time.Now().Add(3 * time.Second)
	timerActive = true
}

func TimerStop() {
	timerActive = false
}

func TimerTimedOut() bool {
	return (timerActive && timerEndTime.Before(time.Now()))
}
