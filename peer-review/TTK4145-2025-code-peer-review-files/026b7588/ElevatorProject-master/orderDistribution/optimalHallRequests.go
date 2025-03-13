package orderDistribution

import (
	"fmt"
	"sort"
	"sync"
	"time"
)

var (
	DoorOpenDuration = 3.0 * time.Second
	travelDuration   = 2500.0 * time.Millisecond
	includeCab       = false
)

type ClearRequestType int

const (
	All ClearRequestType = iota
	InDirn
)

var clearRequestType = InDirn

// Enum definitions
type CallType int

const (
	HallUp CallType = iota
	HallDown
	Cab
)

type HallCallType int

const (
	Up HallCallType = iota
	Down
)

type Dirn int

const (
	DirnDown Dirn = -1
	DirnStop Dirn = 0
	DirnUp   Dirn = 1
)

type ElevatorBehaviour int

const (
	Idle ElevatorBehaviour = iota
	Moving
	DoorOpen
)

type State struct {
	ID    string
	State ElevatorState
	Time  int
}

type RequestSet [][]Request

type ElevatorStates map[string]ElevatorState

// Elevator state structures
type LocalElevatorState struct {
	Behaviour   ElevatorBehaviour
	Floor       int
	Dirn        Dirn
	CabRequests []bool
	Mutex       sync.Mutex
}

type ElevatorState struct {
	Behaviour ElevatorBehaviour
	Floor     int
	Dirn      Dirn
	Requests  [][]bool
	Mutex     sync.Mutex
}

// Request structure
type Request struct {
	Active     bool
	AssignedTo string
}

func NewElevatorState(numFloors int) ElevatorState {
	requests := make([][]bool, numFloors)
	for i := 0; i < numFloors; i++ {
		fmt.Println(requests)
		requests[i] = make([]bool, 3) // Automatisk false
	}

	return ElevatorState{
		Behaviour: Idle,
		Floor:     -1,
		Dirn:      DirnStop,
		Requests:  requests,
	}
}

// Initialize elevator states
func NewLocalElevatorState(e *ElevatorState) LocalElevatorState {

	return LocalElevatorState{
		Behaviour:   e.Behaviour,
		Floor:       e.Floor,
		Dirn:        e.Dirn,
		CabRequests: extractCabRequests(e.Requests),
	}
}

func extractCabRequests(requests [][]bool) []bool {
	var cabRequests []bool
	for _, req := range requests {
		if len(req) > 2 {
			cabRequests = append(cabRequests, req[2])
		}
	}
	return cabRequests
}

// Create ElevatorState from LocalElevatorState
func WithRequests(e *LocalElevatorState, hallReqs [][]bool) ElevatorState {

	var requests [][]bool
	for i, hall := range hallReqs {
		cab := false
		if i < len(e.CabRequests) {
			cab = e.CabRequests[i]
		}
		requests = append(requests, []bool{hall[0], hall[1], cab})
	}
	return ElevatorState{
		Behaviour: e.Behaviour,
		Floor:     e.Floor,
		Dirn:      e.Dirn,
		Requests:  requests,
	}
}

// HallRequests returns hall requests from ElevatorState
func HallRequests(e *ElevatorState) [][]bool {

	hallReqs := make([][]bool, len(e.Requests))
	for i, req := range e.Requests {
		if len(req) >= 2 {
			hallReqs[i] = []bool{req[0], req[1]}
		}
	}
	return hallReqs
}

// Check if there are requests above the current floor
func requestsAbove(e *ElevatorState) bool {

	for i := e.Floor + 1; i < len(e.Requests); i++ {
		if any(reqToBoolArray(e.Requests[i])) {
			return true
		}
	}
	return false
}

// Check if there are requests below the current floor
func requestsBelow(e *ElevatorState) bool {

	for i := 0; i < e.Floor; i++ {
		if any(reqToBoolArray(e.Requests[i])) {
			return true
		}
	}
	return false
}

// Check if there are any active requests
func AnyRequests(e *ElevatorState) bool {

	for _, req := range e.Requests {
		if any(reqToBoolArray(req)) {
			return true
		}
	}
	return false
}

// Check if there are requests at the current floor
func anyRequestsAtFloor(e *ElevatorState) bool {

	return any(reqToBoolArray(e.Requests[e.Floor]))
}

// Determine if elevator should stop at the current floor
func ShouldStop(e *ElevatorState) bool {

	switch e.Dirn {
	case DirnUp:
		return e.Requests[e.Floor][HallUp] || e.Requests[e.Floor][Cab] || !requestsAbove(e) || e.Floor == 0 || e.Floor == len(e.Requests)-1
	case DirnDown:
		return e.Requests[e.Floor][HallDown] || e.Requests[e.Floor][Cab] || !requestsBelow(e) || e.Floor == 0 || e.Floor == len(e.Requests)-1
	case DirnStop:
		return true
	}
	return false
}

// Choose the direction for the elevator to move
func ChooseDirection(e *ElevatorState) Dirn {

	switch e.Dirn {
	case DirnUp:
		if requestsAbove(e) {
			return DirnUp
		} else if anyRequestsAtFloor(e) {
			return DirnStop
		} else if requestsBelow(e) {
			return DirnDown
		}
	case DirnDown, DirnStop:
		if requestsBelow(e) {
			return DirnDown
		} else if anyRequestsAtFloor(e) {
			return DirnStop
		} else if requestsAbove(e) {
			return DirnUp
		}
	}
	return DirnStop
}

// Clear requests at the current floor
func ClearReqsAtFloor(e *ElevatorState, onClearedRequest func(CallType)) {

	for c := 0; c < len(e.Requests[e.Floor]); c++ {
		if e.Requests[e.Floor][c] {
			e.Requests[e.Floor][c] = false
			if onClearedRequest != nil {
				onClearedRequest(CallType(c))
			}
		}
	}
}

// Convert request array to bool array
func reqToBoolArray(req []bool) []bool {
	return req
}

// Check if any request is true
func any(req []bool) bool {
	for _, r := range req {
		if r {
			return true
		}
	}
	return false
}

func isUnassigned(r Request) bool {
	return r.Active && r.AssignedTo == ""
}

func toReq(hallReqs [][]bool) RequestSet {
	reqs := make(RequestSet, len(hallReqs))
	for f, floorReqs := range hallReqs {
		reqs[f] = make([]Request, len(floorReqs))
		for b, req := range floorReqs {
			reqs[f][b] = Request{Active: req, AssignedTo: ""}
		}
	}
	return reqs
}

func initialStates(states ElevatorStates) []State {
	var stateList []State
	for id, state := range states {
		stateList = append(stateList, State{ID: id, State: state, Time: 0})
	}
	sort.Slice(stateList, func(i, j int) bool {
		return stateList[i].ID < stateList[j].ID
	})

	return stateList
}

func OptimalHallRequests(hallReqs [][]bool, elevatorStates ElevatorStates) map[string][][]bool {
	fmt.Println("Hei")
	fmt.Println(elevatorStates)
	reqs := toReq(hallReqs)
	fmt.Println(reqs)
	states := initialStates(elevatorStates)
	fmt.Println(states)

	for i := range states {
		fmt.Println("Hei2")
		performInitialMove(&states[i], &reqs)
	}

	for {
		fmt.Println("Hei3")
		sort.Slice(states, func(i, j int) bool {
			return states[i].Time < states[j].Time
		})

		done := true
		for _, floorReqs := range reqs {
			for _, req := range floorReqs {
				if isUnassigned(req) {
					done = false
					break
				}
			}
		}
		if done {
			break
		}

		performSingleMove(&states[0], &reqs)
	}

	result := make(map[string][][]bool)
	for id := range elevatorStates {
		result[id] = make([][]bool, len(hallReqs))
		for f := range hallReqs {
			result[id][f] = make([]bool, 2)
		}
	}

	for f, floorReqs := range reqs {
		for c, req := range floorReqs {
			if req.Active {
				result[req.AssignedTo][f][c] = true
			}
		}
	}

	return result
}

func performInitialMove(s *State, reqs *RequestSet) {
	switch s.State.Behaviour {
	case DoorOpen:
		s.Time += 2
	case Idle:
		for c := 0; c < 2; c++ {
			if (*reqs)[s.State.Floor][c].Active {
				(*reqs)[s.State.Floor][c].AssignedTo = s.ID
				s.Time += 3
			}
		}
	case Moving:
		s.State.Floor += int(s.State.Dirn)
		s.Time += 5
	}
}

func performSingleMove(s *State, reqs *RequestSet) {
	if s.State.Behaviour == Moving {
		s.State.Floor += int(s.State.Dirn)
		s.Time += 5
	}
}
