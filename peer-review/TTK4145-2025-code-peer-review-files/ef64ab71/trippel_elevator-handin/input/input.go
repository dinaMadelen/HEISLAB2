package input

import (
	"fmt"
	"time"

	"github.com/Eirik-a-Johansen/trippel_elevator/driver"
	"github.com/Eirik-a-Johansen/trippel_elevator/elevator"
	"github.com/Eirik-a-Johansen/trippel_elevator/mergeOrders"
)

/*
This module is responsible for reciving all button presses.
*/
func Recive_buttons(e *elevator.Elevator) {

	ordersEventChannel := make(chan driver.ButtonEvent)
	stopEventChannel := make(chan bool)
	obstructionEventChannel := make(chan bool)

	go driver.PollButtons(ordersEventChannel)
	go driver.PollStopButton(stopEventChannel)
	go driver.PollObstructionSwitch(obstructionEventChannel)

	for {
		select {
		case event := <-ordersEventChannel:
			elevator.Mutex.Lock()

			if event.Button == 2 { //fixes indexing for cab orders
				if !mergeOrders.ID_inList(e.ID, e.Orders[event.Floor][int(event.Button)+e.ID].List) {

					e.Orders[event.Floor][int(event.Button)+e.ID].Value = 1
					e.Orders[event.Floor][int(event.Button)+e.ID].List = append(e.Orders[event.Floor][int(event.Button)+e.ID].List, e.ID)
				}
			} else {
				if !mergeOrders.ID_inList(e.ID, e.Orders[event.Floor][int(event.Button)].List) {

					e.Orders[event.Floor][event.Button].Value = 1
					e.Orders[event.Floor][event.Button].List = append(e.Orders[event.Floor][event.Button].List, e.ID)
				}
			}

			elevator.Mutex.Unlock()

		case stopPressed := <-stopEventChannel:
			elevator.Mutex.Lock()

			if stopPressed {
				fmt.Println("Stop button pressed!")
				// not implemented
			} else {
				fmt.Println("Stop button released!")
				// not implemented
			}

			elevator.Mutex.Unlock()
		case obstruction := <-obstructionEventChannel:
			elevator.Mutex.Lock()

			if obstruction {
				fmt.Println("Obstruction detected!")
				e.DoorObstruction = true
			} else {
				fmt.Println("Obstruction cleared!")
				e.DoorObstruction = false
			}
			elevator.Mutex.Unlock()
		}
		time.Sleep(100 * time.Millisecond)
	}
}
