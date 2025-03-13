package timer

import (
	"time"
)

var (
	timer *time.Timer
	timerActive bool
)

func Start(duration_s float64) {
	if timer == nil {
		timer = time.NewTimer(time.Duration(duration_s) * time.Second)
	} else {
		timer.Reset(time.Duration(duration_s) * time.Second)
	}
	timerActive = true
}

func Stop() {
	if timer != nil {
		timer.Stop()
	}
	timerActive = false
}

func PollTimer(timer_channel chan bool) {
	for {
		if timerActive {
			<-timer.C
			timer_channel <- true
			timerActive = false
		}
	}
}



