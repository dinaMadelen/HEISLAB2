package primaryprocess

import (
	. "Driver-go/elevator"
	. "Driver-go/requests"
	. "Driver-go/utilities"
	"fmt"
	"sync"
)

var (
	//Oversikt over checkpoints som skal forsikre mot packetloss ved sending mellom backup og master
	checkpointHallcallMap = make(map[int]HallCalls)
	//Det siste checkpointet som ble sendt
	latestCheckpoint HallCalls
	//Mutex til checkpoint map
	checkpointHallcallMapMU sync.Mutex
)

// Global hallcall kø H
var H HallCalls

// Mulig unødvendig funksjon for å se om det er bestilt en bestemt hallcall
func PrimaryProcessGetHallCall(dir int, floor int) bool {
	return H.Queue[dir][floor] == 1
}

// Funksjon for å slette hallcalls
func PrimaryProcessDeleteHallCall(dir int, floor int) {
	H.Queue[dir][floor] = 0
}

// Funksjon som skal legge til hallcalls i den globale køen
func PrimaryProcessSaveHallCall(dir int, floor int) {
	//Sjekker om hallcall allerede er bestilt
	if PrimaryProcessGetHallCall(dir, floor) {
		return
	} else {
		//Hvis ikke legges hallcallen til H
		H.Queue[dir][floor] = 1
		//Sender hallcall til sorteringsalgoritmen
		PrimaryProcessAssignHallCall(dir, floor)
		//Sender kopi av hallcalls til backup
		PrimaryProcessSendCopyToBackup()
		return
	}

}

// Funksjon som bestemmer hvilken heis som skal ta hallcall
func PrimaryProcessAssignHallCall(dir int, floor int) {
	//Struct bestående av alle heiser og tiden de bruker på å gjøre hallcall
	elevators := []struct {
		elevator Elevator
		time     int
	}{
		{PrimaryProcessGetElevator("1"), CalculateTimeHallcall(PrimaryProcessGetElevator("1"))},
		{PrimaryProcessGetElevator("2"), CalculateTimeHallcall(PrimaryProcessGetElevator("2"))},
		{PrimaryProcessGetElevator("3"), CalculateTimeHallcall(PrimaryProcessGetElevator("3"))},
	}

	//Finner heisen som bruker minst tid på hallcall
	bestElevator := elevators[0]
	for _, e := range elevators[1:] {
		if e.time < bestElevator.time {
			bestElevator = e
		}
	}

	//Gi hallcall til den raskeste heisen
	bestElevator.elevator.Requests[floor][dir] = 1
}

// Funksjon som oppdaterer latestHallcall
func PrimaryProcessUpdateLatestHallcall(cp int) {
	checkpointHallcallMapMU.Lock()
	defer checkpointHallcallMapMU.Unlock()

	value, exists := checkpointHallcallMap[cp]
	//Fjerner alle utdaterte checkpoints
	if exists {
		latestCheckpoint = value
		for k, _ := range checkpointHallcallMap {
			if k < cp {
				delete(checkpointHallcallMap, k)
			}
		}
	} else {
		fmt.Println("Key not found")
	}
}

// Funksjon for å hente nyeste checkpoint
func PrimaryProcessGetLatestCheckpoint() HallCalls {
	checkpointHallcallMapMU.Lock()
	defer checkpointHallcallMapMU.Unlock()
	return latestCheckpoint
}


const TRAVEL_TIME = 20    //Tiden det tar å bevege seg mellom tgo etasjer
const DOOR_OPEN_TIME = 20 //Tid døren er åpen før geisen forsetter

// Beregner hvor lang tid det tar for en heis å fullføre en hallcall
// Funksjonen simulerer heisens bevegelse
func CalculateTimeHallcall(org Elevator) int {
	e := org //for å være 100% sikker på at simuleringen ikke endrer noe
	duration := 0

	switch e.Behaviour {
	case Idle:
		e.Dirn = RequestsChooseDirection(e).Direction
		if e.Dirn == Stop {
			return duration
		}
	case Moving:
		duration += TRAVEL_TIME / 2
		e.Floor += int(e.Dirn)
	case DoorOpen:
		duration -= DOOR_OPEN_TIME / 2
	}

	for {
		if RequestsShouldStop(e) {
			e = RequestsClearAtCurrentFloor(e) // No side-effects in simulation
			duration += DOOR_OPEN_TIME
			e.Dirn = RequestsChooseDirection(e).Direction
			if e.Dirn == Stop {
				return duration
			}
		}
		e.Floor += int(e.Dirn)
		duration += TRAVEL_TIME
	}
}