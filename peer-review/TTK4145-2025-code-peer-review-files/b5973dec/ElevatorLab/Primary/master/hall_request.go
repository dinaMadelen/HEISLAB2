package master

import (
	"Primary/elevator"
	"Primary/elevator/elevio"
)

// --------------- REQUEST DISTRIBUTION FUNCTIONS --------------- //

// --- GLOBAL FUNCTIONS --- //

func HallRequest_assigner() {
	for {
		
		a := <-elevator.Hall
		temp := 1000
		id_temp := 0

		for i := 0; i < len(elevator.Elevators); i++ {
			if elevator.Request_timeToServe(elevator.Elevators[i], a) < temp {
				temp = elevator.Request_timeToServe(elevator.Elevators[i], a)
				id_temp = i + 1
			}
		}

		println(id_temp)
		var x elevio.OptimalButtonEvent = elevio.OptimalButtonEvent{Floor: a.Floor, Button: elevio.BT_Cab, ElevatorID: id_temp}
		broadcastOptimalEvent(x)
	}
}