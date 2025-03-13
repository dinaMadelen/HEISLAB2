package cost

import (
	"Driver-go/config"
	"Driver-go/elevio"
)

const TRAVEL_TIME = 10
const NumElevators = 4

// Cost function that calculates the cost of assigning an order to an elevator
func Cost(elev *config.ElevatorDistributor, req elevio.ButtonEvent) int {

	if elev.Behaviour  != config.Unavailable {
		e := new(config.ElevatorDistributor)
		*e = *elev
		e.Requests[req.Floor][req.Button] = config.Confirmed

		duration := 0

		switch e.Behaviour {
		case config.Idle:
			requestChooseDirection(e)
			if e.Direction == config.Stop {
				return duration
			}
		case config.Moving:
			duration += TRAVEL_TIME / 2
			e.Floor += int(e.Direction)
		case config.DoorOpen:
			duration -= config.DoorOpenDuration / 2
		}

		for {
			if requestShouldStop(*e) {
				requestClearAtCurrentFloor(e)
				duration += config.DoorOpenDuration
				requestChooseDirection(e)
				if e.Direction == config.Stop {
					return duration
				}
			}
			e.Floor += int(e.Direction)
			duration += TRAVEL_TIME
		}
	}
	return 999
}

func requestsAbove(elev config.ElevatorDistributor) bool {
	for f := elev.Floor + 1; f < config.NumFloors; f++ {
		for btn := range elev.Requests[f] {
			if elev.Requests[f][btn] == config.Confirmed {
				return true
			}
		}
	}
	return false
}

func requestsBelow(elev config.ElevatorDistributor) bool {
	for f := 0; f < elev.Floor; f++ {
		for btn := range elev.Requests[f] {
			if elev.Requests[f][btn] == config.Confirmed {
				return true
			}
		}
	}
	return false
}

func requestClearAtCurrentFloor(elev *config.ElevatorDistributor){
	elev.Requests[elev.Floor][int(elevio.BT_Cab)] = config.None
	switch {
	case elev.Direction  == config.Up:
		elev.Requests[elev.Floor][int(elevio.BT_HallUp)] = config.None
		if !requestsAbove(*elev) {
			elev.Requests[elev.Floor][int(elevio.BT_HallDown)] = config.None
		}
	case elev.Direction == config.Down:
		elev.Requests[elev.Floor][int(elevio.BT_HallDown)] = config.None
		if !requestsBelow(*elev) {
			elev.Requests[elev.Floor][int(elevio.BT_HallUp)] = config.None
		}
	}
}

func requestShouldStop(elev config.ElevatorDistributor) bool {
	switch {
	case elev.Direction  == config.Down:
		return elev.Requests[elev.Floor][int(elevio.BT_HallDown)] == config.Confirmed ||
			elev.Requests[elev.Floor][int(elevio.BT_Cab)] == config.Confirmed ||
			!requestsBelow(elev)
	case elev.Direction == config.Up:
		return elev.Requests[elev.Floor][int(elevio.BT_HallUp)] == config.Confirmed ||
			elev.Requests[elev.Floor][int(elevio.BT_Cab)] == config.Confirmed ||
			!requestsAbove(elev)
	default:
		return true
	}
}

func requestChooseDirection(elev *config.ElevatorDistributor) {
	switch elev.Direction{
	case config.Up:
		if requestsAbove(*elev) {
			elev.Direction  = config.Up
		} else if requestsBelow(*elev) {
			elev.Direction = config.Down
		} else {
			elev.Direction  = config.Stop
		}
	case config.Down:
		fallthrough
	case config.Stop:
		if requestsBelow(*elev) {
			elev.Direction = config.Down
		} else if requestsAbove(*elev) {
			elev.Direction = config.Up
		} else {
			elev.Direction = config.Stop
		}
	}
}


