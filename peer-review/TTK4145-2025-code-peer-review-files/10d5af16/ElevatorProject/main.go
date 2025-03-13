package main

import (
	"Driver-go/elevio"
	"comm"
	"elevator"
	"elevator_io_device"
	"elevator_logic"
	"fmt"
	"fsm"
	"net"
	"time"
	"os"
)

func main() {

	var conn net.Conn

	if len(os.Args) > 1 && os.Args[1] == "slave" {
		conn = comm.Comm_slaveConnectToMaster()
	} else {
		conn = comm.Comm_masterConnectToSlave()
	}

	fmt.Println("Started!")

	// burde vel ikke måtte definere denne på nytt, er jo definert i elevio?
	numFloors := 4
	const maxDuration time.Duration = 1<<63 - 1

	elevio.Init("localhost:15657", numFloors)

	fsm := fsmpkg.FSM{El: elevator.Elevator_uninitialized(), Od: elevator_io_device.Elevio_getOutputDevice()}

	elevator_logic.ElevLogic_runElevator(fsm, maxDuration, conn)
}
