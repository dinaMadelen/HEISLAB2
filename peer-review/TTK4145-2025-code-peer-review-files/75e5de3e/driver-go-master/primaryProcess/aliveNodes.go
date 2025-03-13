package primaryprocess

import (
	"Driver-go/elevator"
	"sync"
	"time"
)

const (
	timeBeforeDead time.Duration = 30 * time.Millisecond
	sleepingTime                 = 30
)

type Worldview struct {
	HeisID    int
	Timestamp time.Time
	requests  [elevator.NFloors][elevator.NBtns]int //burde ordne seg når vi kan import elevator/elevator
	e         elevator.Elevator                     //Inneholder EB, Dirn osv osv..

}

// siden ActiveNodes er public og vi har et multithread program => må vi bruke mutex
var (
	activeNodes = make(map[int]Worldview)
	mu          sync.Mutex
)

func PrimaryProcessUpdateAliveNodes(vb Worldview) {
	mu.Lock()
	defer mu.Unlock()
	activeNodes[vb.HeisID] = vb
}

// når node er død, ping, slik at vi kan reassigne døde hall-calls
func PrimaryProcessPingWhenNodeIsDead(ch chan<- Worldview) {
	for {
		for k, v := range activeNodes {
			if time.Since(v.Timestamp) >= timeBeforeDead { //dersom node er død
				ch <- v                //skriver til channel om at følgende verdensbilde er død
				delete(activeNodes, k) //sletter død node fra map
			}
		}
		time.Sleep(sleepingTime * time.Millisecond)
	}
}

func PrimaryProcessGetElevator(ID int) elevator.Elevator {
	e := activeNodes[ID]
	return e.e
}
