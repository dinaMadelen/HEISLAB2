package orderserver

import (
	"group48.ttk4145.ntnu/elevators/models"
)

func requestsAbove(e localElevator) bool {
	for _, floorRequests := range e.requests[e.Floor+1:] {
		if any(floorRequests[:]) {
			return true
		}
	}
	return false
}

func requestsBelow(e localElevator) bool {
	for _, floorRequests := range e.requests[:e.Floor] {
		if any(floorRequests[:]) {
			return true
		}
	}
	return false
}

func anyRequestsAtFloor(e localElevator) bool {
	for _, request := range e.requests[e.Floor] {
		if request {
			return true
		}
	}
	return false
}

func shouldStop(e localElevator) bool {
	switch e.Direction {
	case models.Up:
		return e.requests[e.Floor][models.HallUp] ||
			e.requests[e.Floor][models.Cab] ||
			!requestsAbove(e) ||
			e.Floor == 0 ||
			e.Floor == len(e.requests)-1
	case models.Down:
		return e.requests[e.Floor][models.HallDown] ||
			e.requests[e.Floor][models.Cab] ||
			!requestsBelow(e) ||
			e.Floor == 0 ||
			e.Floor == len(e.requests)-1
	case models.Stop:
		return true
	}
	return false
}

func chooseDirection(e localElevator) models.MotorDirection {
	switch e.Direction {
	case models.Up:
		if requestsAbove(e) {
			return models.Up
		} else if anyRequestsAtFloor(e) {
			return models.Stop
		} else if requestsBelow(e) {
			return models.Down
		} else {
			return models.Stop
		}
	case models.Down, models.Stop:
		if requestsBelow(e) {
			return models.Down
		} else if anyRequestsAtFloor(e) {
			return models.Stop
		} else if requestsAbove(e) {
			return models.Up
		} else {
			return models.Stop
		}
	}
	return models.Stop
}

func clearReqsAtFloor(e localElevator, onClearedRequest func(models.ButtonType)) localElevator {
	e2 := e

	clear := func(b models.ButtonType) {
		if e2.requests[e2.Floor][b] {
			if onClearedRequest != nil {
				onClearedRequest(b)
			}
			e2.requests[e2.Floor][b] = false
		}
	}
	clear(models.Cab)
	switch e.Direction {
	case models.Up:
		if e2.requests[e2.Floor][models.HallUp] {
			clear(models.HallUp)
		} else if !requestsAbove(e2) {
			clear(models.HallDown)
		}
	case models.Down:
		if e2.requests[e2.Floor][models.HallDown] {
			clear(models.HallDown)
		} else if !requestsBelow(e2) {
			clear(models.HallUp)
		}
	case models.Stop:
		clear(models.HallUp)
		clear(models.HallDown)
	}
	return e2
}
