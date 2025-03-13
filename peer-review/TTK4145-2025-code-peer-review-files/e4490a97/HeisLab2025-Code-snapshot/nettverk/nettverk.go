package nettverk

import (
	"fmt"
	"os"
	"time"

	elev "github.com//HeisLab2025/elev_algo/elevator_io"
	"github.com//HeisLab2025/elev_algo/fsm"
	b "github.com//HeisLab2025/nettverk/network/bcast"
	"github.com//HeisLab2025/nettverk/network/localip"
	"github.com//HeisLab2025/nettverk/network/peers"
)

var ID string
var InfoMap = make(map[string]InformationElev) //is used to gather the data from each elevator before it is sent to the HRA

type ConfirmationState int

const (
	no_call      ConfirmationState = 0
	unregistered ConfirmationState = 1
	registered   ConfirmationState = 2
)

type HelloMsg struct {
	Message string
	Iter    int
}

type InformationElev struct {
	State        HRAElevState
	HallRequests [][2]bool
	ID           string
}

type HRAElevState struct {
	Behavior    string `json:"behaviour"`
	Floor       int    `json:"floor"`
	Direction   string `json:"direction"`
	CabRequests []bool `json:"cabRequests"`
}

type HRAInput struct {
	HallRequests [][2]bool               `json:"hallRequests"`
	States       map[string]HRAElevState `json:"states"`
}

// Gets status from the elevator and sends it on the channel as a informationElev-variabel
func SetElevatorStatus(ch_HRAInputTx chan InformationElev) {
	for {
		info := Converter(fsm.FetchElevatorStatus())
		info.ID = ID
		ch_HRAInputTx <- info
		time.Sleep(1000 * time.Millisecond)
	}
}

func BroadcastElevatorStatus(ch_HRAInputTx chan InformationElev) {
	for {
		b.Transmitter(14000, ch_HRAInputTx)
	}
}

func RecieveElevatorStatus(ch_HRAInputRx chan InformationElev) {
	for {
		b.Receiver(14000, ch_HRAInputRx)
	}
}

// updating peer-list and adding elevators with their data to the InfoMap
func Nettverk_hoved(ch_HRAInputRx chan InformationElev, id string) {

	if id == "" {
		localIP, err := localip.LocalIP()
		if err != nil {
			fmt.Println(err)
			localIP = "DISCONNECTED"
		}
		id = fmt.Sprintf("peer-%s-%d", localIP, os.Getpid())
	}
	ID = id

	peerUpdateCh := make(chan peers.PeerUpdate)
	peerTxEnable := make(chan bool)
	go peers.Transmitter(16000, id, peerTxEnable)
	go peers.Receiver(16000, peerUpdateCh)

	for {
		select {
		case p := <-peerUpdateCh:
			fmt.Printf("Peer update:\n")
			fmt.Printf("  Peers:    %q\n", p.Peers)
			fmt.Printf("  New:      %q\n", p.New)
			fmt.Printf("  Lost:     %q\n", p.Lost)

		case a := <-ch_HRAInputRx:
			InfoMap[a.ID] = a
		}
	}
}

// converting a elev.elevator-variabel to a InformationElev-variabel
func Converter(e elev.Elevator) InformationElev {
	rawInput := e
	hallRequests := make([][2]bool, len(rawInput.Requests))
	cabRequests := make([]bool, len(rawInput.Requests))

	for i := 0; i < len(rawInput.Requests); i++ {
		hallRequests[i] = [2]bool{rawInput.Requests[i][0], rawInput.Requests[i][1]}
		cabRequests[i] = rawInput.Requests[i][2]
	}

	input := InformationElev{
		HallRequests: hallRequests,
		State: HRAElevState{
			Behavior:    stateToString(rawInput.State),
			Floor:       rawInput.Floor,
			Direction:   dirnToString(rawInput.Dirn),
			CabRequests: cabRequests,
		},
	}
	return input
}

// converting states the elev_algo module uses states the HRA uses
func stateToString(s elev.State) string {
	switch s {
	case elev.IDLE:
		return "idle"
	case elev.MOVE:
		return "moving"
	case elev.DOOROPEN:
		return "doorOpen"
	case elev.STOP:
		return "doorOpen"
	default:
		return "idle"
	}
}

// converting directions the elev module uses to directions the HRA uses
func dirnToString(s elev.MotorDirection) string {
	switch s {
	case elev.MD_Up:
		return "up"
	case elev.MD_Down:
		return "down"
	case elev.MD_Stop:
		return "stop"
	default:
		return "stop"
	}
}

// Gets output from HRA and sends to the elev module
func FromHRA(HRAOut chan map[string][][2]bool, ch_elevator_queue chan [][2]bool) {
	for {
		output := <-HRAOut
		for k, v := range output {
			if k == ID {
				ch_elevator_queue <- v
			}
		}
	}
}
