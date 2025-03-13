package timer

import (
	"time"
)

var (
	timerEndTime float64
	timerActive  bool
)

// getWallTime returnerer veggklokken som et flyttall
func getWallTime() float64 {
	now := time.Now()
	return float64(now.Unix()) + float64(now.Nanosecond())*1e-9
}

// timerStart starter en timer med en spesifisert varighet i sekunder
func TimerStart(duration float64) {
	timerEndTime = getWallTime() + duration
	timerActive = true
}

// timerStop stopper den aktive timeren
func TimerStop() {
	timerActive = false
}

// timerTimedOut sjekker om timeren har gått ut
func TimerTimedOut() bool {
	return timerActive && getWallTime() > timerEndTime
}
