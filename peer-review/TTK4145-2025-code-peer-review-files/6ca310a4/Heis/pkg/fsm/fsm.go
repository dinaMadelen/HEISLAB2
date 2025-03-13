package fsm

import (
	"Heis/pkg/elevator"
	"Heis/pkg/elevio"
	"fmt"
)

// define in config
const N_floors = 4
const N_buttons = 3


// FSM handles core logic of a single Elevator. Interacts with orders via localAssignedOrderCH, localRequestCH and completedOrderCH. 
// Also takes input from elevio on drv channels. Interacts with external timer on doorTimerStartCH and doorTimerFinishedCH
func Fsm(elev *elevator.Elevator, drv_buttons chan elevio.ButtonEvent, drv_floors chan int, drv_obstr,
	drv_stop chan bool, doorTimerStartCH chan float64, doorTimerFinishedCH chan bool,
	id string, localAssignedOrderCH, localRequestCH chan elevio.ButtonEvent, completedOrderCH chan elevio.ButtonEvent) {

	if elevio.GetFloor() == -1 {
		initBetweenFloors(elev)
	}

	for {
		select {
		//Inputs (buttons pressed) on each elevator is channeled to their respective local request
		case button_input := <-drv_buttons:
			localRequestCH <- button_input

		//When an assigned order on a local elevator is channeled, it is set as an order to requestButtonPress that makes the elevators move
		case Order := <-localAssignedOrderCH:
			requestButtonPress(elev, Order.Floor, Order.Button, doorTimerStartCH, completedOrderCH)

		case current_floor := <-drv_floors:
			floorArrival(elev, current_floor, doorTimerStartCH, completedOrderCH)

			fmt.Printf("drv_floors: %v", current_floor)
		case obstruction := <-drv_obstr:
			if obstruction {
				(*elev).Obstructed = true
			} else {
				(*elev).Obstructed = false
				doorTimerStartCH <- (*elev).Config.DoorOpenDuration_s
			}

		case <-doorTimerFinishedCH:
			if !elev.Obstructed {
				DoorTimeout(elev, doorTimerStartCH, completedOrderCH)
				fmt.Println("drv_doortimer timed out")
				DoorTimeout(elev, doorTimerStartCH, completedOrderCH)
			}

		}
	}
}
