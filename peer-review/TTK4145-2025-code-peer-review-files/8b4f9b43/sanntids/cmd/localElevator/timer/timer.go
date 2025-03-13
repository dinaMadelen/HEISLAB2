package timer

import (
	"time"
)


var timeoutChan = make(chan bool)
var doorTimer *time.Timer
const (
	TRAVEL_TIME = 10
	DOOR_OPEN_TIME = 3
)

func TimerStart(duration float64) {
	if doorTimer != nil {
		doorTimer.Stop()
	}
	doorTimer = time.AfterFunc(time.Duration(duration)*time.Second, func() {
		timeoutChan <- true
	})
}

func TimerReset(duration float64) {
	if doorTimer == nil {
		TimerStart(duration)
		return
	}
	if !doorTimer.Reset(time.Duration(duration) * time.Second) {
		TimerStart(duration)
	}
}

func TimerStop() {
	if doorTimer != nil {
		doorTimer.Stop()
	}
}

func TimeoutChan() <-chan bool {
	return timeoutChan
}


