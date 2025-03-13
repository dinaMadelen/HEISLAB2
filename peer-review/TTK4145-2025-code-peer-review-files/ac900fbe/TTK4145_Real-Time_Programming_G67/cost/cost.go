package cost

import(
	. "Driver-go/types"
	"Driver-go/requests"
)

/*
TimeToServeRequest simulates the time it takes for an elevator to serve a request.
The function takes an elevator and a channel of ButtonEvents as input, and returns the time it takes to serve the request.
The function is blocking, and will wait for a ButtonEvent to be received on the channel before starting the simulation.
*/

func TimeToServeRequest(e_old Elevator, receivedCh <-chan ButtonEvent) float64 {
	e := e_old
	buttonEvent := <-receivedCh
	b := buttonEvent.Button
	f := buttonEvent.Floor

	e.Requests[f][b] = true
	arrivedAtRequest := false

	ifEqual := func(inner_b ButtonType, inner_f int) {
		if inner_b == b && inner_f == f {
			arrivedAtRequest = true
		}
	}

	duration := 0.0

	switch e.Behaviour {
	case ElevatorBehaviour(EB_Idle):
		e.Dirn = requests.RequestsChooseDirection(e).Dirn
		if e.Dirn == MD_Stop {
			return duration
		}
	case ElevatorBehaviour(EB_Moving):
		duration += e.Config.TimeBetweenFloors / 2
		e.Floor += int(e.Dirn)
	case ElevatorBehaviour(EB_DoorOpen):
		duration -= e.Config.DoorOpenDuration / 2
	}

	for {
		if requests.RequestsShouldStop(e) {
			e = requests.RequestsClearAtCurrentFloor(e, ifEqual)
			if arrivedAtRequest {
				return duration
			}
			duration += e.Config.DoorOpenDuration
			e.Dirn = requests.RequestsChooseDirection(e).Dirn
		}
		e.Floor += int(e.Dirn)
		duration += e.Config.TimeBetweenFloors
	}
}