package watchdog

import(
	"time"
)

func Watchdog(seconds int, ch_reset chan bool, ch_signal chan bool){
	watchdogTimer := time.NewTimer(time.Duration(seconds) * time.Second)

	for {
		select{
		case <- ch_reset:
			watchdogTimer.Reset(time.Duration(seconds) * time.Second)

		case <- ch_signal:
			ch_reset <- true
			watchdogTimer.Reset(time.Duration(seconds) * time.Second)
		}
	}
}