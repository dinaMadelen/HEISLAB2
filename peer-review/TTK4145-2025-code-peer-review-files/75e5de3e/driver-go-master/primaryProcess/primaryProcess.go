package primaryprocess

import (
	. "Driver-go/elevator"
	. "Driver-go/elevio"
	. "Driver-go/fsm"
	. "Driver-go/timer"
	. "Driver-go/utilities"
	"fmt"
	"net"
	"os"
	"sync"
	"time"
)

const (
	PrimaryProcessSendAddr       = "255.255.255.255:22022"      //Adresse Master sender
	PrimaryProcessReceiveAddr    = ":22021"                     //Adresse Master lytter på
	PrimaryProcessHeartbeatMsg   = `{"Tag": "HeartbeatMaster"}` //Melding master sender
	HeartbeatSleep = 500                          				//ms mellom hver heartbeat
)

var (
	checkpoint   = 0        //Hvilke versjon av verdensbilde har backup
	checkpointMU sync.Mutex //checkpoint vil aksesseres av flere forskjellige threads.
)

func PrimaryProcess() {
	//INIT
	FsmInit()

	if GetFloor() == -1 {
		Fsm_onInitBetweenFloors()
	}

	//channels
	drv_buttons := make(chan ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	drv_timer := make(chan bool)
	TimerInit(drv_timer)
	MasterListenerChannel := make(chan string) //Skriver melding på denne channel når det kommer en ny

	//Go-routines
	go PollButtons(drv_buttons)
	go PollFloorSensor(drv_floors)
	go PollObstructionSwitch(drv_obstr)
	go PollStopButton(drv_stop)
	go PrimaryProcessMasterSendHeartbeat()
	go PrimaryProcessMasterListener(MasterListenerChannel)

	for {
		select {
		//Single elevator->
		case a := <-drv_floors:
			fmt.Print("Arrived on floor ", a)
			FsmOnFloorArrival(a)

		case a := <-drv_buttons:
			msg, _ := UtilitiesJsonButtonPress(Tag("ButtonPress"), a.Floor, int(a.Button))
			PrimaryProcessDistributeMsg(msg)
			fmt.Printf("%+v\n", a)
			//Midlertidig ordning for lys 
			SetButtonLamp(a.Button, a.Floor, true)
			fmt.Println("this is a buttonpress ", a.Floor)
			//Hvis knappetrykk er cabcall i primary process
			if a.Button==BT_Cab {
			Fsm_onRequestButtonPress(a.Floor, a.Button)}

		case a := <-drv_obstr:
			fmt.Printf("%+v\n", a)
			if a {
				SetMotorDirection(MD_Stop)
			} else {
				SetMotorDirection(MD_Up)
			}

		case a := <-drv_stop:
			fmt.Printf("%+v\n", a)
			for f := 0; f < NFloors; f++ {
				for b := ButtonType(0); b < 3; b++ {
					SetButtonLamp(b, f, false)
				}
			}
		case <-drv_timer:
			TimerStop()
			FsmOnDoorTimeout()
			fmt.Print("go timedout")


		case a := <-MasterListenerChannel:
			PrimaryProcessDistributeMsg(a)

		}
		time.Sleep(100 * time.Millisecond)
	}
}

func PrimaryProcessIncrementCheckpoint() {
	checkpointMU.Lock()
	defer checkpointMU.Unlock()
	checkpoint += 1
}

func PrimaryProcessReadCheckpoint() int {
	checkpointMU.Lock()
	defer checkpointMU.Unlock()
	return checkpoint
}

// Lytter til meldinger, og sender den på channel
func PrimaryProcessMasterListener(ch chan string) {

	//UDP
	udpAddr, err := net.ResolveUDPAddr("udp", PrimaryProcessReceiveAddr)
	if err != nil {
		fmt.Printf("Error resolving address: %v\n", err)
		os.Exit(1)
	}

	conn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		fmt.Printf("Error creating UDP connection: %v\n", err)
		os.Exit(1)
	}
	defer conn.Close()

	fmt.Printf("Listening for messages on %s...\n", PrimaryProcessReceiveAddr)

	//Buffer
	buffer := make([]byte, 1024)

	//Lytter til meldinger
	for {
		numBytesReceived, _, err := conn.ReadFromUDP(buffer)
		if err != nil {
			fmt.Printf("Error receiving data: %v\n", err)
			continue
		}

		message := string(buffer[:numBytesReceived])
		ch <- message

	}
}

// Master sender heartbeat ved jevne mellomrom
func PrimaryProcessMasterSendHeartbeat() {
	SendUDPAddr, err := net.ResolveUDPAddr("udp", PrimaryProcessSendAddr)
	if err != nil {
		fmt.Println(err)
		return
	}
	conn, err := net.DialUDP("udp", nil, SendUDPAddr)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer conn.Close()

	for {
		cp := PrimaryProcessReadCheckpoint()
		msg := fmt.Sprintf(`{"Tag": "HeartbeatMaster", "Checkpoint":%d}`, cp)
		_, err := conn.Write([]byte(msg))
		if err != nil {
			fmt.Println("Primary failed to send heartbeat:", err)
			return
		}
		time.Sleep(HeartbeatSleep * time.Millisecond)

	}
}

