package orderserver

import (
	"log"
	"sort"
	"time"

	"group48.ttk4145.ntnu/elevators/models"
)

type State struct {
	models.ElevatorState
	CabRequests []bool
	time        time.Time
}

type Request struct {
	active     bool
	assignedTo int
}

const (
	unassignedId = -1 // an Id to represent that a request is not assigned to any elevator
)

func optimalHallRequests(elevators elevators) map[models.Id]models.Orders {
	log.Printf("[orderserver] Calculating optimal orders: %v", elevators)

	reqs := addRequests(elevators)
	states := initialStates(elevators)

	for i := range states {
		performInitialMove(&states[i], &reqs)
	}
	for {
		// Sort states by time
		sort.Slice(states, func(i, j int) bool {
			return states[i].time.Before(states[j].time)
		})
		done := true
		if anyUnassigned(reqs) {
			done = false
		}
		if unvisitedAreImmediatelyAssignable(reqs, states) {
			assignImmediate(&reqs, &states)
			done = true
		}
		if done {
			break
		}
		performSingleMove(&states[0], &reqs)
	}
	results := make(map[models.Id]models.Orders)
	for _, state := range states {
		results[models.Id(state.Id)] = make(models.Orders, numFloors)
		for i, reqsAtFloor := range reqs {
			for j, req := range reqsAtFloor {
				if req.active && req.assignedTo == int(state.Id) {
					results[models.Id(state.Id)][i][j] = true
				}
			}
		}
		for i, cabRequest := range state.CabRequests {
			if cabRequest {
				results[models.Id(state.Id)][i][models.Cab] = true
			}
		}
	}
	return results
}

// addRequests creates a 2D array of requests from the hall buttons
func addRequests(e elevators) [][2]Request {
	// initialize a 2D array of requests, one for each floor and direction
	reqs := make([][2]Request, numFloors)
	for i := range reqs {
		reqs[i] = [2]Request{
			{active: false, assignedTo: unassignedId},
			{active: false, assignedTo: unassignedId},
		}
	}
	// add the requests from the hall buttons
	for f, floorRequests := range e.hallRequests {
		for c, req := range floorRequests {
			reqs[f][c] = Request{
				active:     req,
				assignedTo: unassignedId,
			}
		}
	}
	return reqs
}

func initialStates(e elevators) []State {
	states := make([]State, len(e.states))
	i := 0
	for _, elevator := range e.states {
		states[i] = State{
			ElevatorState: elevator.ElevatorState,
			CabRequests:   elevator.cabRequests,
			time:          time.Now(),
		}
		i++
	}
	return states
}

func performInitialMove(s *State, req *[][2]Request) {
	switch s.Behavior {
	case models.DoorOpen: // if the elevator is at a floor with the door open, wait for it to close
		s.time = s.time.Add(doorOpenDuration / 2)
		s.Behavior = models.Idle
		fallthrough
	case models.Idle: // if the elevator is idle, check if there are any requests at the current floor
		for c := range 2 {
			if (*req)[s.Floor][c].active {
				(*req)[s.Floor][c].assignedTo = int(s.Id)
				s.time = s.time.Add(doorOpenDuration)
			}
		}
	case models.Moving:
		s.Floor += int(s.Direction)
		s.time = s.time.Add(travelDuration / 2)
	}
}

func performSingleMove(s *State, req *[][2]Request) {
	//add a elevator with all the unassisgned requests
	e := anyUnassignedElevator(s, req)

	onClearRequest := func(c models.ButtonType) {
		switch c {
		case models.HallUp, models.HallDown:
			(*req)[s.Floor][c].assignedTo = int(s.Id)
		case models.Cab:
			s.CabRequests[s.Floor] = false
		}
	}

	switch s.Behavior {
	case models.Moving:
		if shouldStop(e) {
			s.Behavior = models.DoorOpen
			s.time = s.time.Add(doorOpenDuration)
			clearReqsAtFloor(e, onClearRequest)
		} else {
			s.Floor += int(s.Direction)
			s.time = s.time.Add(travelDuration)
		}
	case models.Idle, models.DoorOpen:
		s.Direction = chooseDirection(e)
		if s.Direction == models.Stop {
			if anyRequestsAtFloor(e) {
				clearReqsAtFloor(e, onClearRequest)
				s.time = s.time.Add(doorOpenDuration)
				s.Behavior = models.DoorOpen
			} else {
				s.Behavior = models.Idle
			}
		} else {
			s.Behavior = models.Moving
			s.time = s.time.Add(travelDuration)
			s.Floor += int(s.Direction)
		}
	}
}

func anyUnassignedElevator(s *State, req *[][2]Request) localElevator {
	e := localElevator{
		ElevatorState: s.ElevatorState,
		requests:      make([][3]bool, numFloors),
	}

	for f, floorRequests := range *req {
		for c, req := range floorRequests {
			if req.active && req.assignedTo == unassignedId {
				e.requests[f][c] = true
			}
		}
	}
	for s, cabRequest := range s.CabRequests {
		if cabRequest {
			e.requests[s][models.Cab] = true
		}
	}

	return e
}

func unvisitedAreImmediatelyAssignable(reqs [][2]Request, states []State) bool {
	for _, state := range states {
		if any(state.CabRequests) {
			return false
		}
	}
	for f, reqsAtFloor := range reqs {
		activeCount := 0
		for _, req := range reqsAtFloor {
			if req.active {
				activeCount++
			}
		}
		if activeCount == 2 {
			return false
		}
		for _, req := range reqsAtFloor {
			if req.active && req.assignedTo == unassignedId {
				found := false
				for _, state := range states {
					if state.Floor == f && !any(state.CabRequests) {
						found = true
						break
					}
				}
				if !found {
					return false
				}
			}
		}
	}
	return true
}

func assignImmediate(reqs *[][2]Request, states *[]State) {
	for f, reqsAtFloor := range *reqs {
		for c, req := range reqsAtFloor {
			for i := range *states {
				s := &(*states)[i]
				if req.active && req.assignedTo == unassignedId {
					if s.Floor == f && !any(s.CabRequests) {
						(*reqs)[f][c].assignedTo = int(s.Id)
						s.time = s.time.Add(doorOpenDuration)
					}
				}
			}
		}
	}
}

func any(arr []bool) bool {
	for _, req := range arr {
		if req {
			return true
		}
	}
	return false
}

func anyUnassigned(reqs [][2]Request) bool {
	for _, floorReqs := range reqs {
		for _, req := range floorReqs {
			if req.active && req.assignedTo == unassignedId {
				return true
			}
		}
	}
	return false
}
