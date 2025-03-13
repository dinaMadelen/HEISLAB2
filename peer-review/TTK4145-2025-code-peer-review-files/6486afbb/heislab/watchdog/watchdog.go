package main

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/network/bcast"
)

const (
	WatchdogPort  = 15500
	RestartTimeMs = 5000
)

func main() {
	id := os.Args[1:][0]
	ID, _ := strconv.Atoi(id)
	alive := make(chan bool)

	go bcast.Receiver(WatchdogPort+ID, alive)
	for {
		select {
		case <-alive:
			fmt.Println("Watchdog received alive signal")
		case <-time.After(RestartTimeMs * time.Millisecond):
			goto restart
		}
	}
restart:
	cmd := exec.Command("gnome-terminal", "exec", "./restart.sh", id, "5590"+id)
	cmd.Start()
	time.Sleep(RestartTimeMs * time.Millisecond)
	os.Exit(0)
}
