package timer

import (
	"time"
)

/*
Open door for 3 seconds

make sure movement is not to slow
*/

func Timer() float64 {
	now := time.Now()
	return float64(now.Unix()) + float64(now.Nanosecond())*1e-9
}

var timerEndTime float64
var timerActive bool

func Timer_start(duration float64) {
	timerEndTime = Timer() + duration
	timerActive = true
}

func Timer_stop() {
	timerActive = false
}

func Timer_TimedOut() bool {
	return timerActive && Timer() > timerEndTime
}
