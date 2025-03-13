package timer

import (	
	"time"
	"fmt"
)

func UpdateTimer(timer *time.Timer, timerStart_chan chan time.Duration, stop_chan chan bool) {
	for{
		select{
		case duration:= <-timerStart_chan:
			fmt.Println("duration: ", duration)
			timer.Reset(duration)
		case <- stop_chan:
			return
		}
	
	}
}


