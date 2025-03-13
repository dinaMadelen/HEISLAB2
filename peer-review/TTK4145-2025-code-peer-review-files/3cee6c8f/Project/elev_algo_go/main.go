package main

import (
	"Driver-go/elevio"
	"fmt"
	"time"

	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/elevator"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/fsm"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/hub_algo"
	"github.com/anonym/TTK4145-Project/Project/elev_algo_go/timer"
	"github.com/anonym/TTK4145-Project/elev_algo_go/elevator_network"
)

var hubIP string
var worldView elevator.ElevWorldView

func getLocalIP() (string, error) {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return "", err
	}
	for _, addr := range addrs {
		if ipNet, ok := addr.(*net.IPNet); ok && !ipNet.IP.IsLoopback() {
			if ipNet.IP.To4() != nil {
				return ipNet.IP.String(), nil
			}
		}
	}
	return "", fmt.Errorf("no IP address found")
}


func startHubAlgo() error {
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "linux":
		cmd = exec.Command("gnome-terminal", "--", "go", "run", "main.go")
	case "windows":
		cmd = exec.Command("cmd", "/C", "start", "powershell", "go", "run", "main.go")
	default:
		panic("OS not supported")
	}
	return cmd.Start()
}

func main() {
	elevio.Init("localhost:15657", elevator.NumFloors)

	localIP, err := getLocalIP()
	if err != nil {
		fmt.Println("Error getting local IP:", err)
		return
	}

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors  := make(chan int)
	drv_obstr   := make(chan bool)
	drv_stop    := make(chan bool)
	timer_ch    := make(chan bool)
	

	go elevio.PollButtons(drv_buttons)
	go elevio.PollFloorSensor(drv_floors)
	go elevio.PollObstructionSwitch(drv_obstr)
	go elevio.PollStopButton(drv_stop)
	go timer.PollTimer(timer_ch)
	
	go func() {
		worldView = UpdateElevWorldView()
		newPacket := elevator_network.WorldviewToPacket(worldView)
		elevator_network.SendWorldview(newPacket)

		time.Sleep(100 * time.Millisecond)
	}()



	fsm.Init()
	initialFloor := elevio.GetFloor()
	if initialFloor == -1 {
		fsm.OnInitBetweenFloors()
	}

	// TODO: Route BtnPress to hub_algo, listen for orders from hub_algo

	for {
		select {
		case a := <-drv_buttons:
			fmt.Printf("%+v\n", a)
			//m.OnRequestButtonPress(a.Floor, a.Button)
			elevator_network.SendButtonEvent(a)
		case a := <-drv_floors:
			fmt.Printf("%+v\n", a)
			if a == -1 {
				//fsm.OnInitBetweenFloors()
			} else {
				fsm.OnFloorArrival(a)
			}

		case a := <-drv_obstr:
			fmt.Printf("%+v\n", a)
			fsm.OnObstruction(a)

		case a := <-drv_stop:
			fmt.Printf("%+v\n", a)
			for f := 0; f < elevator.NumFloors; f++ {
				for b := elevio.ButtonType(0); b < 3; b++ {
					elevio.SetButtonLamp(b, f, false)
				}
			}
		// Listen on order channel
		//case order := <-orderChannel:
		//	fsm.OnReceivedOrder(order)

		case <-timer_ch:
			timer.Stop()
			fsm.OnDoorTimeout()
		}
	}
}
