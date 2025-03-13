package main

import (
	"OneElevator/elevio"
	"OneElevator/network/bcast"
	"OneElevator/network/localip"
	"OneElevator/network/peers"
	"flag"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"
)

type Heartbeat struct {
	Message string
	Iter    int
}

type Order struct {
	id        int
	masterID  int // en master per ordre, kan være ulike mastere for simultane ordre
	currState int
}

type Distributor struct {
	id              int
	elevator        Elevator
	orders          map[int]Order
	elevatorsOnline [3]bool // Array med 3 plasser for å indikere om hver heis er online
}

var (
	Port    int
	id      int
	idStr   string
	portStr string
)

func main() {
	// Initialiser kommandolinje-argumenter
	driverPort := flag.Int("port", 15657, "<-- Default value, override with command line argument -port=xxxxx")
	elevatorId := flag.Int("id", 1, "<-- Default value, override with command line argument -id=x")
	flag.Parse()

	Port = *driverPort
	id = *elevatorId
	portStr = strconv.Itoa(Port)

	// Generer en ID basert på IP hvis id=0
	if id == 0 {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		idStr = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
		fmt.Println("Peer ID:", id)
	} else {
		idStr = strconv.Itoa(id)
	}

	// Initialiser distributor
	distributor := Distributor{
		id:              id,
		elevatorsOnline: [3]bool{false, false, false},
		orders:          make(map[int]Order),
	}

	// Opprett kanaler for nettverkskommunikasjon
	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	helloTx := make(chan Heartbeat)
	helloRx := make(chan Heartbeat)

	// Initialiser nettverksmoduler
	go peers.Transmitter(15647, id, peerTxEnable)
	go peers.Receiver(15647, peerUpdateCh)
	go bcast.Transmitter(16569, helloTx)
	go bcast.Receiver(16569, helloRx)

	// Aktiver peer-oppdagelse
	peerTxEnable <- true

	// Start heartbeat-sending
	go func() {
		heartbeat := Heartbeat{idStr + " is Alive!", 0}
		for {
			heartbeat.Iter++
			helloTx <- heartbeat
			time.Sleep(1 * time.Second)
		}
	}()

	// Start primærprosessen asynkront
	numFloors := 4
	go primaryProcess(portStr, numFloors)

	fmt.Println("System started")

	// Hovedløkke for å håndtere nettverksmeldinger
	for {
		select {
		case p := <-peerUpdateCh:
			fmt.Printf("Peer status:\n")
			fmt.Printf("  Peers:    %q\n", p.Peers)
			fmt.Printf("  New:      %q\n", p.New)
			fmt.Printf("  Lost:     %q\n", p.Lost)
			distributor.updateElevatorsOnline(p)

		case a := <-helloRx:
			fmt.Printf("Received heartbeat: %#v\n", a)
			distributor.updateElevatorsOnlineFromHeartbeat(a)
		}
	}
}

// primaryProcess håndterer hardware-relaterte hendelser for heisen
func primaryProcess(driverPort string, numFloors int) {
	elevio.Init("localhost:"+driverPort, numFloors) // Connect to hardware server

	initElevator(numFloors, elevio.NumButtonTypes)

	// Event channels for hardware events
	var ButtonPressCh = make(chan elevio.ButtonEvent)
	var FloorSensorCh = make(chan int)
	var StopButtonCh = make(chan bool)
	var ObstructionSwitchCh = make(chan bool)

	// Start polling goroutines
	go elevio.PollButtons(ButtonPressCh)
	go elevio.PollFloorSensor(FloorSensorCh)
	go elevio.PollStopButton(StopButtonCh)
	go elevio.PollObstructionSwitch(ObstructionSwitchCh)

	SingleElevator(ButtonPressCh, FloorSensorCh, StopButtonCh, ObstructionSwitchCh)
}

// updateElevatorsOnline oppdaterer listen over tilkoblede heiser basert på peer-oppdateringer
func (d *Distributor) updateElevatorsOnline(p peers.PeerUpdate) {
	// Legg til nye peers
	for _, newPeer := range p.New {
		// Konverter rune til string først
		peerStr := string(newPeer)
		peerInt, err := strconv.Atoi(peerStr)
		if err != nil {
			log.Printf("Warning: Failed to convert newPeer '%s' to int: %v", peerStr, err)
			continue
		}

		// Kontroller at ID er gyldig (1-3) og juster til array-indeks (0-2)
		if peerInt >= 1 && peerInt <= 3 {
			arrayIndex := peerInt - 1
			d.elevatorsOnline[arrayIndex] = true
			log.Printf("Added elevator with ID %d to online list (index %d)", peerInt, arrayIndex)
		} else {
			log.Printf("Warning: Peer ID %d out of range (1-3)", peerInt)
		}
	}

	// Fjern tapte peers
	for _, lostPeer := range p.Lost {
		peerInt, err := strconv.Atoi(lostPeer)
		if err != nil {
			log.Printf("Warning: Failed to convert lostPeer '%s' to int: %v", lostPeer, err)
			continue
		}

		// Kontroller at ID er gyldig (1-3) og juster til array-indeks (0-2)
		if peerInt >= 1 && peerInt <= 3 {
			arrayIndex := peerInt - 1
			d.elevatorsOnline[arrayIndex] = false
			log.Printf("Removed elevator with ID %d from online list (index %d)", peerInt, arrayIndex)
		} else {
			log.Printf("Warning: Peer ID %d out of range (1-3)", peerInt)
		}
	}

	// Skriv ut status for alle heiser
	fmt.Println("Current elevator status:")
	for i := 0; i < 3; i++ {
		status := "OFFLINE"
		if d.elevatorsOnline[i] {
			status = "ONLINE"
		}
		fmt.Printf("Elevator %d: %s\n", i+1, status)
	}
}

// updateElevatorsOnlineFromHeartbeat oppdaterer når en heis sist var aktiv basert på heartbeat
func (d *Distributor) updateElevatorsOnlineFromHeartbeat(hb Heartbeat) {
	// Trekk ut ID fra heartbeat-meldingen (forventet format: "peer-[IP]-[PID] is Alive!")
	parts := strings.Split(hb.Message, " ")
	if len(parts) < 1 {
		log.Printf("Warning: Invalid heartbeat message format: %s", hb.Message)
		return
	}

	peerIdStr := parts[0]
	var idInt int
	var err error

	// Check if it's in the format "peer-XXX-YYY"
	if strings.HasPrefix(peerIdStr, "peer-") {
		// Extract the PID (last part after the last "-")
		lastDashIndex := strings.LastIndex(peerIdStr, "-")
		if lastDashIndex == -1 || lastDashIndex == len(peerIdStr)-1 {
			log.Printf("Warning: Invalid peer ID format (missing process ID): %s", peerIdStr)
			return
		}

		pidStr := peerIdStr[lastDashIndex+1:]
		idInt, err = strconv.Atoi(pidStr)
		if err != nil {
			log.Printf("Warning: Failed to convert PID '%s' to int: %v", pidStr, err)
			return
		}
	} else {
		// Try direct conversion for backward compatibility
		idInt, err = strconv.Atoi(peerIdStr)
		if err != nil {
			log.Printf("Warning: Cannot parse elevator ID from '%s': %v", peerIdStr, err)
			return
		}
	}

	// Map the PID to a valid elevator ID range (1-3)
	// Using modulo to ensure it falls within range
	elevatorId := (idInt % 3) + 1
	arrayIndex := elevatorId - 1

	d.elevatorsOnline[arrayIndex] = true
	log.Printf("Updated heartbeat for elevator %d (index %d) from peer ID: %s (process ID: %d)",
		elevatorId, arrayIndex, peerIdStr, idInt)
}
