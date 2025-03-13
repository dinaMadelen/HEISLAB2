package main

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

func timerStart(duration float64) {
	timerEndTime = getWallTime().Add(time.Duration(duration * float64(time.Second)))
	timerActive = true
}

func timerStop() {
	timerActive = false
}

func timerTimedOut() bool {
	return timerActive && getWallTime().After(timerEndTime)
}
