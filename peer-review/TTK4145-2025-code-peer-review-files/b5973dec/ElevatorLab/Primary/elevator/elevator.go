package elevator

import (
	"Primary/elevator/config"
	"Primary/elevator/elevio"
	"fmt"
	"log"
	"time"
)

// --------------- ELEVATOR FUNCTIONS --------------- //

// --- LOCAL FUNCTIONS --- //

func elevator_initialize() Elevator {

	el := Elevator{Floor: -1,
		Dirn:      elevio.MD_Stop,
		Behaviour: config.EB_Idle,
		Config: config.Config{
			ClearRequestVariant: config.CV_InDirn,
			DoorOpenDuration_s:  config.DoorOpenTime},
	}

	return el
}

func elevator_setAllLights(ele *elevio.Ele, es []Elevator) {

	bu := []elevio.ButtonType{elevio.BT_HallUp, elevio.BT_HallDown, elevio.BT_Cab}

	for floor := 0; floor < config.NumFloors; floor++ {
		for _, B := range bu {
			ele.SetButtonLamp(B, floor, es[ele.ID-1].Request[floor][B] == true)
		}
	}
}

// --- GLOBAL FUNCTIONS --- //

func Elevator_init(addr string, numFloors, elevatorID int, optimal chan elevio.OptimalButtonEvent) {
	ele, err := elevio.NewEle(elevatorID, addr, numFloors)
	if err != nil {
		log.Println(err)
	}
	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)
	timout := make(chan bool)

	go ele.PollButtons(drv_buttons)
	go ele.PollFloorSensor(drv_floors)
	go ele.PollObstructionSwitch(drv_obstr)
	go ele.PollStopButton(drv_stop)
	go timer_poll(ele, timout)
	log.Printf("Started polling sensors for Elevator %d\n", elevatorID)

	inputPollRateMs := 25
	if ele.GetFloor() == -1 {
		fsm_onInitBetweenFloors(ele)
	}

	for {
		select {
		case a := <-optimal:
			fmt.Println("hei")
			if a.ElevatorID == ele.ID {
				fsm_onRequestButtonPress(ele, a.Floor, elevio.ButtonType(a.Button))
			}

		case a := <-drv_buttons:
			log.Printf("Elevator %d received ButtonEvent: %+v\n", elevatorID, a)
			if a.Button == elevio.BT_HallUp || a.Button == elevio.BT_HallDown {
				fmt.Println("Denne kjører")
				Hall <- a
			} else {
				fsm_onRequestButtonPress(ele, a.Floor, elevio.ButtonType(a.Button))
			}

		case a := <-drv_floors:
			fsm_onFloorArrival(ele, a)
			log.Printf("Elevator %d floor sensor event: %+v\n", elevatorID, a)

		case a := <-timout:
			if a {
				timer_stop(ele)
				fsm_onDoorTimeout(ele)
			}
		}

		state.Elevator_id = elevatorID
		state.Elevator_floor = Elevators[elevatorID-1].Floor
		state.Elevator_dir = int(Elevators[elevatorID-1].Dirn)
		state.Elevator_request = Elevators[elevatorID-1].Request
		state.Elevator_behaviour = int(Elevators[elevatorID-1].Behaviour)
		//network.SendElevatorState("3000","localhost", state)
		time.Sleep((500 * time.Duration(inputPollRateMs)))
	}
}

func Elevator_toString(eb config.ElevatorBehaviour) string {
	if eb == config.EB_Idle {
		return "EB_Idle"
	} else if eb == config.EB_DoorOpen {
		return "EB_DoorOpen"
	} else if eb == config.EB_Moving {
		return "EB_Moving"
	} else {
		return "EB_UNDEFINED"
	}
}
