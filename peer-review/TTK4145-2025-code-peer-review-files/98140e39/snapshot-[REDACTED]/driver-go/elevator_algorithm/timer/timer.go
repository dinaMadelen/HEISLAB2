package timer

import "time"

var (
	pollRate  = 20 * time.Millisecond
	startTime time.Time
	active    bool
)

func StartTimer() {
	startTime = time.Now()
	active = true
}

func StopTimer() {
	active = false
}

func PollTimer(receiver chan<- bool, timeOut time.Duration) {
	prev := false

	for {
		time.Sleep(pollRate)
		timedOut := active && time.Since(startTime) > timeOut
		if timedOut && timedOut != prev {
			receiver <- true
		}
		prev = timedOut
	}
}

