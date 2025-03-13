package slave

import "time"

/*
	Timer function that starts a timer for a given amount of time

Input: The channel to start the timer, the timer to be started
*/
func timer(t_start chan int, t_end *time.Timer) {
	for a := range t_start {
		t_end.Reset(time.Second * time.Duration(a))
	}
}
