package elevator

import (
	"Primary/elevator/elevio"
	"time"
)

// --------------- TIMER FUNCTIONS --------------- //

// --- LOCAL FUNCTIONS --- //

func getWallTime() float64 {
	tid := time.Now()
	return float64(tid.UnixNano()) / 1e9
}

func timer_start(ele *elevio.Ele, duration float64) {
	Elevators[ele.ID-1].TimerEndTime = getWallTime() + duration
	Elevators[ele.ID-1].TimerActive = 1
}

func timer_stop(ele *elevio.Ele) {
	Elevators[ele.ID-1].TimerActive = 0
}

func timer_timedOut(ele *elevio.Ele) bool {
	if Elevators[ele.ID-1].TimerActive == 1 && (getWallTime() > Elevators[ele.ID-1].TimerEndTime) {
		return true
	} else {
		return false
	}

}

func timer_poll(ele *elevio.Ele, reciver chan<- bool) {
	prev := false
	for {
		time.Sleep(elevio.PollRate)
		v := timer_timedOut(ele)
		if v != prev {
			reciver <- v
		}
		prev = v
	}
}
