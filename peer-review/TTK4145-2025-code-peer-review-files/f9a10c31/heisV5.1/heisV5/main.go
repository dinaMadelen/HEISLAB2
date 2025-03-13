package main

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"time"

	"heisV5/config"
	"heisV5/elevator"
	"heisV5/elevio"
	"heisV5/fsm"
	"heisV5/hallassigner"
	"heisV5/network/bcast"
	"heisV5/network/peers"
	"heisV5/requests"
	"heisV5/synchronizer"
)

var Port int
var id string

func main() {
	// **Hent heis-ID og port fra kommandolinjen**
	port := flag.Int("port", 15657, "Port for communication")
	elevatorId := flag.String("id", "", "ID of this elevator (string format)")
	flag.Parse()

	Port = *port
	if *elevatorId == "" {
		*elevatorId = fmt.Sprintf("%d", os.Getpid()%100) // **Genererer ID hvis ikke spesifisert**
	}
	id = *elevatorId

	// **Konverter ID fra string til int**
	elevatorID, err := strconv.Atoi(id)
	if err != nil {
		fmt.Println("[ERROR] Could not convert elevator ID to int:", err)
		elevatorID = 0 // Setter en standardverdi hvis konvertering feiler
	}

	// **Initialiser heis og sensorer**
	elevio.Init("localhost:"+strconv.Itoa(Port), 4)
	floorSensor := make(chan int)
	stopButton := make(chan bool)
	obstruction := make(chan bool)

	go elevio.PollFloorSensor(floorSensor)
	go elevio.PollStopButton(stopButton)
	go elevio.PollObstructionSwitch(obstruction)

	// **Initialiser nettverkskommunikasjon**
	peerUpdateCh := make(chan peers.PeerUpdate)
	transmitEnable := make(chan bool)
	go peers.Receiver(Port, peerUpdateCh)
	go peers.Transmitter(Port, id, transmitEnable)

	// **Initialiser kanaler for synkronisering**
	stateUpdateC := make(chan elevator.ElevatorState)
	confirmedStateC := make(chan synchronizer.SystemState)
	newOrderC := make(chan elevio.ButtonEvent)           // Bestillinger fra knapper
	convertedOrderC := make(chan elevator.ElevatorState) // Konverterte bestillinger
	networkTx := make(chan synchronizer.SystemState)
	networkRx := make(chan synchronizer.SystemState)
	deliveredOrderC := make(chan elevio.ButtonEvent, config.BufferSize)
	requests.InitRequests(deliveredOrderC)

	go bcast.Receiver(config.BcastPortNumber, networkRx)
	go bcast.Transmitter(config.BcastPortNumber, networkTx)

	// **Konverter `elevio.ButtonEvent` til `elevator.Elevator` før sending til FSM**
	go func() {
		for btnEvent := range newOrderC {
			convertedOrderC <- elevator.ElevatorState{
				CurrentFloor: btnEvent.Floor,
				Direction:    elevio.MD_Stop, // Standardverdi, kan oppdateres senere
			}
		}
	}()

	// **Start synkronisering av bestillinger og heisstatus**
	go synchronizer.RunStateSynchronizer(
		confirmedStateC,
		newOrderC,
		stateUpdateC,
		networkTx,
		networkRx,
		peerUpdateCh,
		elevatorID, // Sender en `int` i stedet for `string`
	)

	// **Start heisens tilstandsmaskin (FSM)**
	fsm.InitFSM(stateUpdateC, convertedOrderC) // Bruker konvertert kanal

	// **Systemoppstart: Sjekk om vi er i en etasje**
	if floor := elevio.GetFloor(); floor != -1 {
		fmt.Println("Starting at floor:", floor)
		//fsm.OnFloorArrival(floor, fsm.GetElevatorState())
		//Linjen over er nok ikke riktig så lagde en ny under
		fsm.OnFloorArrival(floor)
	} else {
		fmt.Println("Starting between floors, moving to nearest floor...")
		fsm.OnObstructionChange(true) // **Simulerer at heisen skal finne nærmeste etasje**
	}

	fmt.Println("System started!")

	// **Hovedløkken som håndterer bestillinger fra `SystemState`**
	for {
		select {
		case updatedSystemState := <-confirmedStateC:
			// **Bruk `hallassigner` for å fordele bestillinger**
			assignedOrders := hallassigner.RunHRA(updatedSystemState, elevatorID)

			// **Oppdater fsm med de nye bestillingene**
			fsm.handleUpdatedOrders(assignedOrders)

		case newFloor := <-floorSensor:
			fsm.OnFloorArrival(newFloor)

		case peerUpdate := <-peerUpdateCh:
			fmt.Println("Peer update:", peerUpdate)
			if len(peerUpdate.Peers) > 1 {
				transmitEnable <- true
			} else {
				transmitEnable <- false
			}

		case <-stopButton:
			fmt.Println("[WARNING] EMERGENCY STOP PRESSED!")

		case obstructed := <-obstruction:
			fsm.OnObstructionChange(obstructed)
		}

		time.Sleep(10 * time.Millisecond)
	}
}
