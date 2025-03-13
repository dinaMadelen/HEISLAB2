package main

import (
	"flag"
	"fmt"
	"os/exec"
	"strconv"
	"time"

	"github.com/Kirlu3/Sanntid-G30/heislab/backup"
	"github.com/Kirlu3/Sanntid-G30/heislab/config"
	"github.com/Kirlu3/Sanntid-G30/heislab/network/bcast"

	"github.com/Kirlu3/Sanntid-G30/heislab/driver-go/elevio"
	"github.com/Kirlu3/Sanntid-G30/heislab/slave"
)

// the program should be called with go run heislab/main.go -id=x -port=5590x
func main() {
	// id := os.Args[1:][0]
	id := flag.String("id", "inv", "id of this elevator")
	serverPort := flag.String("port", "15657", "port to communicate with elevator")
	flag.Parse()

	if *id == "inv" {
		panic("please specify an id in [0, N_Elevators) with -id=x")
	}

	serverAddress := fmt.Sprintf("localhost:%s", *serverPort)
	elevio.Init(serverAddress, config.N_FLOORS)


	masterToSlaveOfflineCh := make(chan [config.N_ELEVATORS][config.N_FLOORS][config.N_BUTTONS]bool)
	slaveToMasterOfflineCh := make(chan slave.EventMessage)

	go slave.Slave(*id, masterToSlaveOfflineCh, slaveToMasterOfflineCh)
	go backup.Backup(*id, masterToSlaveOfflineCh, slaveToMasterOfflineCh)

	// Watchdog
	ID, _ := strconv.Atoi(*id)
	alive := make(chan bool)
	go bcast.Transmitter(config.WatchdogPort+ID, alive)
	cmd := exec.Command("gnome-terminal", "--", "go", "run", "heislab/watchdog/watchdog.go", *id)
	err := cmd.Start()
	if err != nil {
		fmt.Println(err)
		panic(err)
	}

	for {
		time.After(500 * time.Millisecond)
		alive <- true
	}
}
