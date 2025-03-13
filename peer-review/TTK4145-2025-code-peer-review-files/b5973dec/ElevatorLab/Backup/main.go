package main

import (
	"Backup/data_transfer"
)

func main() {
	go data_transfer.ReciveElevatorState("3000")
	go data_transfer.ReciveHeartBeat("4000", "localhost")
	select {}

}
