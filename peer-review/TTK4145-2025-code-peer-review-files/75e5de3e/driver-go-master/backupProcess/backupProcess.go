package backupProcess

import (
	. "Driver-go/elevator"
	. "Driver-go/elevio"
	. "Driver-go/fsm"
	. "Driver-go/primaryProcess"
	. "Driver-go/timer"
	. "Driver-go/utilities"
	"fmt"
	"net"
	"time"
)

const (
	BPSendAddr          = "255.255.255.255:22021" //Adresse backup sender på
	BPReceiveAddr       = ":22022"                //Adresse backup lytter på
	BPHeartbeatMsg      = `{"Tag": "HeartbeatBackup"}`
	HeartbeatFromMaster = `{"Tag": "HeartbeatMaster"}` //Heartbeat melding som master sender, til sammenligning
)

// Global variabel for å lagre hall calls i backup
var BPHallCalls HallCalls

func BackupProcess() {

	//INIT
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
	drv_SendHeartbeat := make(chan bool)     //Denne blir sann hver gang backup skal sende heartbeat
	drv_listenToHeartbeat := make(chan bool) //Når 3x heartbeat mangler fra Master, blir denne channelen sann

	go PollButtons(drv_buttons)
	go PollFloorSensor(drv_floors)
	go PollObstructionSwitch(drv_obstr)
	go PollStopButton(drv_stop)
	go UtilitiesSendHeartbeat(drv_SendHeartbeat)
	go BackupProcessListenToMessages(drv_listenToHeartbeat)

	//Hoved-løkka for Backup
	for {
		select {

		//Single elevator->
		case a := <-drv_floors:
			fmt.Print("Arrived on floor ", a)
			FsmOnFloorArrival(a)

		case a := <-drv_buttons:
			fmt.Printf("%+v\n", a)
			SetButtonLamp(a.Button, a.Floor, true)
			fmt.Println("this is a buttonpress ", a.Floor)
			Fsm_onRequestButtonPress(a.Floor, a.Button)

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

		//Bytter til primary når Master dør
		case <-drv_listenToHeartbeat:
			PrimaryProcess()

		//Backup sender Heartbeat melding
		case <-drv_SendHeartbeat:
			UtilitiesSendMessage(BPHeartbeatMsg, BPSendAddr)

		}
	}
}

// Denne lytter til meldinger og agerer deretter
func BackupProcessListenToMessages(ch chan<- bool) {

	//Internett greier
	BPReceiveUDPAddr, err := net.ResolveUDPAddr("udp", BPReceiveAddr)
	if err != nil {
		fmt.Println(err)
		return
	}
	conn, err := net.ListenUDP("udp", BPReceiveUDPAddr)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer conn.Close()

	for {
		//Buffer for å motta meldinger på
		buffer := make([]byte, 1024)

		//Dette er en bug. Per nå vil backup bare bli master hvis den ikke leser noen meldinger på 5x heartbeats.
		conn.SetReadDeadline(time.Now().Add(HeartbeatSleep * 5 * time.Millisecond))

		//Leser meldinger til buffer
		n, _, err := conn.ReadFromUDP(buffer)

		if err != nil {
			if e, ok := err.(net.Error); ok && e.Timeout() {
				fmt.Println("Backup did not receive heartbeat, becoming primary.")
				conn.Close()
				ch <- true // Ping til channel at Master er død
				return
			} else {
				fmt.Println("Error reading from UDP:", err)
				return
			}
		}

		msg := string(buffer[:n])

		//Sjekker om melding er en heartbeat
		if msg[:len(HeartbeatFromMaster)] == HeartbeatFromMaster {
			fmt.Println("Heartbeat received")
			checkpoint := 3 //denne er midlertidig
			BackupProcessAcknowledgeMaster(checkpoint)

			receivedMessage, err := UtilitiesRecieveJsonString(msg)
			if err != nil {
				fmt.Println("Feil ved parsing av heartbeat-melding:", err)
				return

			}

			BackupProcessStoreHallCalls(receivedMessage.HallCalls)

		}

	}
}

// Kopierer hallcalls fra master til BPHallCalls
func BackupProcessStoreHallCalls(hallcalls HallCalls) {
	for floor := 0; floor < NFloors; floor++ {
		for button := 0; button < NBtns; button++ {
			BPHallCalls.Queue[floor][button] = hallcalls.Queue[floor][button]
		}
	}
	fmt.Println("Backup oppdaterte hall calls:", BPHallCalls)
}

// Når Backup mottar hall-calls fra master, responder med hvilke checkpoint
func BackupProcessAcknowledgeMaster(checkpoint int) {
	msg := fmt.Sprintf(`{"Tag": "Acknowledgement", "Checkpoint":%d}`, checkpoint)
	UtilitiesSendMessage(msg, BPSendAddr)
}
