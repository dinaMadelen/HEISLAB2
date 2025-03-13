package elevator

import (
	"time"
)

// Variabler for å holde styr på timeren
var timerEndTime time.Time
var timerActive bool

// TimerStart starter timeren med angitt varighet
func TimerStart(duration float64) {
	timerEndTime = time.Now().Add(time.Duration(duration) * time.Second)
	timerActive = true
}

// TimerStop stopper timeren
func TimerStop() {
	timerActive = false
}

// TimerTimedOut sjekker om timeren har utløpt
func TimerTimedOut() bool {
	return timerActive && time.Now().After(timerEndTime)
}
