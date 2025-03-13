package elevator

import "time"

var (
	pollRate  = 20 * time.Millisecond
	timeOut   = 3 * time.Second
	timeOfStart time.Time
	timerActive    bool
)

func StartTimer() {
	timeOfStart = time.Now()
	timerActive = true
}

func StopTimer() {
    timerActive = false
}

func PollTimer(receiver chan<- bool) {
	prev := false
	for {
		time.Sleep(pollRate)
		timedOut := timerActive && time.Since(timeOfStart) > timeOut
		if timedOut && timedOut != prev {
			receiver <- true
		}
		prev = timedOut
	}
}

func TimedOut() bool {
	return timerActive && time.Since(timeOfStart) > timeOut
}
