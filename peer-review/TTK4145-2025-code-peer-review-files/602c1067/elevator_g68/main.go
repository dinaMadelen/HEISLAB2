package main

import (
	"elevator/elevio"
	"flag"
	"fmt"
	"net"
)

func main() {
	addrPtr := flag.String("addr", "localhost:15657", "Address of elevator hardware")
	flag.Parse()

	numFloors := 9
	elevio.Init(*addrPtr, numFloors)

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)
	go broadcastReceiver()

	conUDP, _ := net.Dial("udp", "255.255.255.255:15000")

	for {
		select {
		case a := <-drv_buttons:
			// broadcast via UDP
			// receive vis UDP as well
			// TODO setup broadcast address all servers can write to and read from
			fmt.Printf("Button %+v\n", a)
			conUDP.Write([]byte{65, 66, 67, 68}) // WriteToUDP
		case a := <-drv_floors:
			fmt.Printf("Floors %+v\n", a)
		case a := <-drv_obstr:
			fmt.Printf("Obstruction %+v\n", a)
		case a := <-drv_stop:
			fmt.Printf("Stop %+v\n", a)
		}
	}
}

func broadcastReceiver() {
	addr, _ := net.ResolveUDPAddr("udp", ":15000")
	conn, _ := net.ListenUDP("udp", addr)
	defer conn.Close()

	var buf [16]byte
	for {
		n, _ := conn.Read(buf[:]) // readFromUDP
		fmt.Println(buf[:n])
	}
}

// elevator := NewElevator(*addrPtr, 9, 1000*time.Millisecond)
// elevator.Run()
// }
