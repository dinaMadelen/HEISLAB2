package fsm

import (
	"Driver-go/elevio"
	"fmt"
	"math"
)

type DirnBehaviourPair struct {
	dirn      Dirn
	behaviour ElevatorBehaviour
}

func requests_above(elevator Elevator) bool {
	for i := elevator.Floor + 1; i < NumFloors; i++ {
		for j := 0; j < NumButtons; j++ {
			if elevator.Requests[i][j] {
				return true
			}
		}
	}
	return false
}

func requests_below(elevator Elevator) bool {
	for i := 0; i < elevator.Floor; i++ {
		for j := 0; j < NumButtons; j++ {
			if elevator.Requests[i][j] {
				return true
			}
		}
	}
	return false
}

func requests_here(elevator Elevator) bool {
	for j := 0; j < NumButtons; j++ {
		if elevator.Requests[elevator.Floor][j] {
			return true
		}
	}
	return false
}

func requests_chooseDirection(e Elevator) DirnBehaviourPair {
	switch e.Dirn {
	case D_Up:
		if requests_above(e) {
			return DirnBehaviourPair{D_Up, EB_Moving}
		} else if requests_here(e) {
			return DirnBehaviourPair{D_Stop, EB_DoorOpen}
		} else if requests_below(e) {
			return DirnBehaviourPair{D_Down, EB_Moving}
		} else {
			return DirnBehaviourPair{D_Stop, EB_Idle}
		}
	case D_Down:
		if requests_below(e) {
			return DirnBehaviourPair{D_Down, EB_Moving}
		} else if requests_here(e) {
			return DirnBehaviourPair{D_Stop, EB_DoorOpen}
		} else if requests_above(e) {
			return DirnBehaviourPair{D_Up, EB_Moving}
		} else {
			return DirnBehaviourPair{D_Stop, EB_Idle}
		}
	case D_Stop:
		if requests_here(e) {
			return DirnBehaviourPair{D_Stop, EB_DoorOpen}
		} else if requests_above(e) {
			return DirnBehaviourPair{D_Up, EB_Moving}
		} else if requests_below(e) {
			return DirnBehaviourPair{D_Down, EB_Moving}
		} else {
			return DirnBehaviourPair{D_Stop, EB_Idle}
		}
	default:
		return DirnBehaviourPair{D_Stop, EB_Idle}
	}
}

func requests_shouldStop(e Elevator) bool {
	switch e.Dirn {
	case D_Down:
		return e.Requests[e.Floor][elevio.BT_HallDown] ||
			e.Requests[e.Floor][elevio.BT_Cab] ||
			!requests_below(e)
	case D_Up:
		return e.Requests[e.Floor][elevio.BT_HallUp] ||
			e.Requests[e.Floor][elevio.BT_Cab] ||
			!requests_above(e)
	case D_Stop:
		fallthrough
	default:
		return true
	}
}

// Denne funksjonen sjekker om heisen er i en etasje og om det er en knapp som er trykket inn i den etasjen. Dersom det er det, vil den returnere true.
func requests_shouldClearImmediately(e Elevator, btn_floor int, btn_type elevio.ButtonType) bool {
	return e.Floor == btn_floor &&
		((e.Dirn == D_Up && btn_type == elevio.BT_HallUp) ||
			(e.Dirn == D_Down && btn_type == elevio.BT_HallDown) ||
			e.Dirn == D_Stop ||
			btn_type == elevio.BT_Cab)
}

func requests_clearAtCurrentFloor(e Elevator) Elevator {
	e.Requests[e.Floor][elevio.BT_Cab] = false
	switch e.Dirn {
	case D_Up:
		if !requests_above(e) && !e.Requests[e.Floor][elevio.BT_HallUp] {
			e.Requests[e.Floor][elevio.BT_HallDown] = false
		}
		e.Requests[e.Floor][elevio.BT_HallUp] = false
	case D_Down:
		if !requests_below(e) && !e.Requests[e.Floor][elevio.BT_HallDown] {
			e.Requests[e.Floor][elevio.BT_HallUp] = false
		}
		e.Requests[e.Floor][elevio.BT_HallDown] = false

	case D_Stop:
		fallthrough

	default:
		e.Requests[e.Floor][elevio.BT_HallUp] = false
		e.Requests[e.Floor][elevio.BT_HallDown] = false
	}
	return e
}

func Calculate_priority_value(button_event elevio.ButtonEvent) int {
	request_floor := button_event.Floor
	//request_button := button_event.Button

	//button point dir. minus is down
	//requst_button_point_dir := -1
	//if int(request_button) == 0 {
	//requst_button_point_dir = 1
	//}

	elevator := GetElevatorStruct()
	NumFloors_minus_1 := NumFloors - 1
	//Calculate how much this elevator wants this request.
	priority_value := 2 * 10 * NumFloors_minus_1 // max value

	//Bad version, it calculate absolute distance
	delta_floor := request_floor - elevator.Floor
	//fmt.Println("---------------------------------------------------")
	//fmt.Println("request floor: ", request_floor)
	//fmt.Println("elevator floor: ", elevator.Floor)
	//fmt.Println("Delta floor: ", delta_floor)
	//fmt.Println("Priority value :", priority_value)

	priority_value = priority_value - int(math.Abs(float64(delta_floor)))*10

	//The code beneath did not work, hence it is commeted out.
	/*
		//antall etasjer unna gir -10 poeng
		//delta_floor gir minus value if requested floor is below
		delta_floor := request_floor - elevator.Floor

		//if elevator dosen ot have a moving dirn
		if int(elevator.Dirn) == 0 {
			priority_value = priority_value - int(math.Abs(float64(delta_floor)))*10

			//HVis heis beveger seg mot request, og request peker i samme retning som heisens bevegelse
		} else if math.Copysign(1, float64(delta_floor)) == math.Copysign(1, float64(elevator.Dirn)) {
			// delta_floor have the same sign as elv direction. Meaning it is going towards the request)

			if math.Copysign(1, float64(requst_button_point_dir)) == math.Copysign(1, float64(elevator.Dirn)) {
				//Elevator moves towards request and in same direction as request
				priority_value = priority_value - int(math.Abs(float64(priority_value)-float64(delta_floor)))*10

			} else {
				//Elevator moves toward request, but request is in oposit riection
				if int(elevator.Dirn) < 0 {
					//elevator moves downward
					wortcase_down := elevator.Floor + request_floor //goes to bottom, and turns direciton and moves up
					priority_value = priority_value - wortcase_down*10
				} else {
					//elevator moves up
					worstcase_up := (NumFloors_minus_1 - elevator.Floor) + (NumFloors_minus_1 - request_floor)
					priority_value = priority_value - worstcase_up*10
				}
			}

		} else {
			// Elevator is not going towards the request

			if math.Copysign(1, float64(requst_button_point_dir)) == math.Copysign(1, float64(elevator.Dirn)) {
				//Elevator will not point in same drection after turn at a worstcase top

			} else {
				//Elevator moves toward request, but request is in oposit riection
				if int(elevator.Dirn) < 0 {
					//elevator moves downward
					nr_of_floors_traveled := elevator.Floor + NumFloors_minus_1 + request_floor
					priority_value = priority_value - nr_of_floors_traveled*10
				} else {
					//elevator moves up
					nr_of_floors_traveled := (NumFloors_minus_1 - elevator.Floor) + NumFloors_minus_1 + request_floor
					priority_value = priority_value - nr_of_floors_traveled*10

				}
			}

		}*/
	fmt.Println("Priority value:", priority_value)
	//fmt.Println("---------------------------------------------------")
	return priority_value
}
