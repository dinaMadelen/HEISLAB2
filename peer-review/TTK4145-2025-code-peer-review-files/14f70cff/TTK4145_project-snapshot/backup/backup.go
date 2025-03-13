package backup

import (
	"elevator/distribution"
	"elevator/elevio"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"os"
	"os/exec"
	"runtime"
	"time"
)

const bufSize = 1024

func Backup() elevio.Elevator {
	fmt.Println("Starting backup")
	conn := distribution.DialBroadcastUDP(28574)

	buffer := make([]byte, bufSize)
	primaryAlive := true
	var elevator elevio.Elevator

	for primaryAlive {
		conn.SetReadDeadline(time.Now().Add(3 * time.Second))
		n, _, err := conn.ReadFrom(buffer)

		if err != nil {
			if errors.Is(err, os.ErrDeadlineExceeded) {
				primaryAlive = false
			}
		} else {
			var msg elevio.Elevator
			json.Unmarshal(buffer[:n], &msg)
			elevator = msg
		}
	}
	if (elevio.Elevator{}) == elevator {
		return elevio.Elevator{Floor: -1, Dirn: elevio.MD_Stop, Behaviour: elevio.EB_Idle}
	}
	return elevator
}

func TransformToPrimary(ch chan elevio.Elevator, sim_port string) {
	switch runtime.GOOS {
	case "linux":
		exec.Command("gnome-terminal", "--", "go", "run", "main.go", "--port", sim_port).Run()
	case "windows":
		exec.Command("wt.exe", "wsl", "go", "run", "main.go", "--port", sim_port).Run()
	default:
		panic("OS not supported")
	}

	fmt.Println("I am promoted:)")

	conn := distribution.DialBroadcastUDP(28574)
	addr, _ := net.ResolveUDPAddr("udp4", fmt.Sprintf("255.255.255.255:%d", 28574))

	for {
		msg := <-ch

		jsonstr, _ := json.Marshal(msg)

		if len(jsonstr) > bufSize {
			panic(fmt.Sprintf(
				"Tried to send a message longer than the buffer size (length: %d, buffer size: %d)\n\t'%s'\n"+
					"Either send smaller packets, or go to network/bcast/bcast.go and increase the buffer size",
				len(jsonstr), bufSize, string(jsonstr)))
		}
		conn.WriteTo(jsonstr, addr)
	}

}
