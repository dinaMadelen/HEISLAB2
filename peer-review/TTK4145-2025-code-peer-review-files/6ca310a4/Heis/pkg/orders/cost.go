package orders

import (
	"Heis/pkg/elevator"
	"Heis/pkg/elevio"
	"Heis/pkg/fsm"
)

const TRAVEL_TIME = 10
const NumElevators = 4

func cost(e elevator.Elevator, req elevio.ButtonEvent) int {
	if elevator.ElevatorBehaviour(e.Behaviour) != elevator.EB_Unavailable {

		duration := 0

		switch e.Behaviour {
		case elevator.EB_Idle:
			pair := fsm.ChooseDirection(e)
			e.Dirn = pair.Dirn
			e.Behaviour = pair.Behaviour
			if e.Dirn == elevio.MD_Stop {
				return duration //Dersom EB_IDLE, og hvis det er ingen retning, blir det ingen ekstra kostnad

			}
		case elevator.EB_Moving:
			duration += TRAVEL_TIME / 2 //dersom heisen er i beveglse legger vi til en kostand
			e.Floor += int(e.Dirn)
		case elevator.EB_DoorOpen:
			duration -= int(e.Config.DoorOpenDuration_s) / 2
			//Trekker fra kostnad siden heisen allerede står i ro med dørene åpne og er dermed:
			//Klar til å ta imot nye bestillinger på denne etasjoen, uten ekstra (halvparten) ventetid for å åpne dører

		}
		for {
			if fsm.ShouldStop(e) {
				e = clearAtCurrentFloor(e)
				duration += int(e.Config.DoorOpenDuration_s)
				// might not need this?
				pair := fsm.ChooseDirection(e)
				e.Dirn = pair.Dirn
				e.Behaviour = pair.Behaviour
				// ...
				if e.Dirn == elevio.MD_Stop {
					return duration //returner duration når den simulerte heisen har kommet til en stopp
				}
			}
			e.Floor += int(e.Dirn)  //Hvis det ikke er kommet noe tegn på at den stopper sier vi at den estimerte heisen sier vi her at den går til en ny etasje
			duration += TRAVEL_TIME //da vil vi også legge til en TRAVEL_TIME kostand for denne opeerasjonen
		}

	}
	return 999 //returnerer høy kostnad dersom heisen er EB_unavailable
}



//version without sending on completed channel to orders
func clearAtCurrentFloor(e elevator.Elevator) elevator.Elevator {
	switch e.Config.ClearRequestVariant {
	case elevator.CV_ALL:
		for btn := 0; btn < N_buttons; btn++ {
			e.LocalOrders[e.Floor][btn] = false
		}
	case elevator.CV_InDirn:
		e.LocalOrders[e.Floor][elevio.BT_Cab] = false
		switch e.Dirn {
		case elevio.MD_Up:
			if (!fsm.Above(e)) && (e.LocalOrders[e.Floor][elevio.BT_HallUp] == false) {
				e.LocalOrders[e.Floor][elevio.BT_HallDown] = false
			}
			e.LocalOrders[e.Floor][elevio.BT_HallUp] = false
		case elevio.MD_Down:
			if (!fsm.Below(e)) && (e.LocalOrders[e.Floor][elevio.BT_HallDown] == false) {
				e.LocalOrders[e.Floor][elevio.BT_HallUp] = false
			}
			e.LocalOrders[e.Floor][elevio.BT_HallDown] = false
		// case elevio.MD_Stop:
		// 	e.LocalOrders[e.Floor][elevio.BT_HallUp] = false
		// 	e.LocalOrders[e.Floor][elevio.BT_HallDown] = false
		// 	e.LocalOrders[e.Floor][elevio.BT_Cab] = false
		default:
			e.LocalOrders[e.Floor][elevio.BT_HallUp] = false
			e.LocalOrders[e.Floor][elevio.BT_HallDown] = false
			//e.LocalOrders[e.Floor][elevio.BT_Cab] = false
		}
	default:

	}
	return e
}
