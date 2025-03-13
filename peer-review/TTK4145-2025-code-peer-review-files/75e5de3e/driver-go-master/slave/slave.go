package slave

import (
	. "Driver-go/elevio"
	. "Driver-go/fsm"
	. "Driver-go/timer"
	. "Driver-go/utilities"
	. "Driver-go/primaryProcess"
	"fmt"
	"time"
)

func SlaveProcess() {

	//INIT
	numFloors := 4
	Init("localhost:15657", numFloors)
	FsmInit()

	if GetFloor() == -1 {
		Fsm_onInitBetweenFloors()
	}

	//Channels
	drv_buttons := make(chan ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	drv_timer := make(chan bool)
	TimerInit(drv_timer)
	drv_heartbeat := make(chan bool)

	//Go-routines
	go PollButtons(drv_buttons)
	go PollFloorSensor(drv_floors)
	go PollObstructionSwitch(drv_obstr)
	go PollStopButton(drv_stop)
	go UtilitiesSendHeartbeat(drv_heartbeat)

	//Hoved-løkke for Slave
	for {
		select {
		case a := <-drv_floors:
			fmt.Print("Arrived on floor ", a)
			FsmOnFloorArrival(a)

		case a := <-drv_buttons:
			fmt.Printf("%+v\n", a)
			SetButtonLamp(a.Button, a.Floor, true)
			fmt.Println("this is a buttonpress ", a.Floor)
			if a.Button == 0 || a.Button == 1 { //hall call
				msg, _ := UtilitiesJsonButtonPress("Button", a.Floor, int(a.Button))
				fmt.Print(msg)
				UtilitiesSendMessage(msg, PrimaryProcessReceiveAddr)

			} else {
				Fsm_onRequestButtonPress(a.Floor, a.Button)
			}

		case a := <-drv_obstr:
			fmt.Printf("%+v\n", a)
			if a {
				SetMotorDirection(MD_Stop)
			} else {
				SetMotorDirection(MD_Up)
			}

		case a := <-drv_stop:
			fmt.Printf("%+v\n", a)
			for f := 0; f < numFloors; f++ {
				for b := ButtonType(0); b < 3; b++ {
					SetButtonLamp(b, f, false)
				}
			}
		case <-drv_timer:
			TimerStop()
			FsmOnDoorTimeout()
			fmt.Print("go timedout")
			//PSUEDO-kode til å motta hall call fra master:
			//case a:=<- fromMaster:  hvis mottatt hall call fra master, sett på lys, og evt add til requests
			//if a.elevator==this elevator{
			//fsm_onRequestButtonPress(a.Floor, a.Button)}
			//else{kun sett på lys}

			time.Sleep(time.Duration(400) * time.Millisecond)

		case <-drv_heartbeat:
			//fmt.Print("heartbeat")
			//dialToMaster() //TODO

		}

	}
}
