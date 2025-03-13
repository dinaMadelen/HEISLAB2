package main

import (
	"Driver-go/connectivity"
	"Driver-go/elevio"
	"Driver-go/fsm"
	"fmt"
	"time"
)

const (
	NUMFLOORS       = 4
	Port_server_id0 = 15657
)

func main() {

	port := Port_server_id0 + connectivity.ID
	ip := fmt.Sprintf("localhost:%d", port)
	fmt.Println("ID: ", connectivity.ID, ", ip: ", ip)
	elevio.Init(ip, NUMFLOORS)

	//var d elevio.MotorDirection = elevio.MD_Up
	//elevio.SetMotorDirection(d)

	//--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

	//connectivity.TCP_setup()
	TCP_receive_channel := make(chan connectivity.Worldview_package)
	//TCP_send_channel_listen := make(chan connectivity.Worldview_package)
	//TCP_send_channel_dail := make(chan connectivity.Worldview_package)
	go connectivity.TCP_receving_setup(TCP_receive_channel)

	// Go routine to send world view every second
	var world_view_send_ticker <-chan time.Time
	ticker := time.NewTicker(1000 * time.Millisecond)
	defer ticker.Stop() // Ensure the ticker stops when the program exits
	world_view_send_ticker = ticker.C

	//--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
	//Order setup
	order_to_send_chan := make(chan connectivity.DoneProcessedOrder)
	connectivity.Order_setup(order_to_send_chan)
	//--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

	drv_buttons := make(chan elevio.ButtonEvent)
	drv_floors := make(chan int)
	drv_obstr := make(chan bool)
	drv_stop := make(chan bool)

	// Channel to receive timer timeout events
	timerTimeoutChan := make(chan bool)

	go elevio.PollButtons(drv_buttons)
	//søker igjennom alle etasjene og sjekker alle typer knapper for den etasjen.
	//Den sjekker ved å sende en tpc getbutton(etasje, knappetype) og får tilbake true/false. Dersom dette er anderledes enn fra forigje gang den sjekket og den nå er nå true.
	//Skriver den til chanelen drv_buttons. Den skriver da en ButtonEvent (struct) med etasje og knappetype.
	go elevio.PollFloorSensor(drv_floors)
	//Sjekker om heisen er i en etasje og den etasjen er ulik det den var sist gang den sjekket. Dersom den er det, skriver den til chanelen drv_floors. Den skriver da etasjen heisen er i, i form av en int.
	go elevio.PollObstructionSwitch(drv_obstr)
	//Sjekker om det er en obstruction i heisen. Dersom statuesn på obstruction endrer seg så skriver den true til chanelen drv_obstr. Dersom det ikke er obstruction, skriver den false.
	go elevio.PollStopButton(drv_stop)
	//Sjekker om stopknappen er trykket inn. Den vil skrive true eller false til chanelen drv_stop når statusen endrer seg.

	go fsm.PollTimerTimeout(timerTimeoutChan)

	//--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

	time.Sleep(2000 * time.Millisecond)
	fmt.Println("Started!")

	//inputPollRateMs := 25
	prev_floor := -1

	fsm.SetElevatorToValidStartPossition()

	for {
		select {
		// Kan enten få inn en ButtonEvent, en etasje (int) eller en obstruction
		case button_event := <-drv_buttons: // Hvis det kommer en ButtonEvent {Floor, ButtonType} fra chanelen drv_buttons
			fmt.Println("Button event-------------------------------------------------------------------------")
			fmt.Printf("%+v\n", button_event)

			if len(connectivity.Get_all_online_ids()) != 1 && button_event.Button != elevio.BT_Cab {
				connectivity.PrintIsOnline()
				//This is start the prosses of finding the best elevator, only if there are other elevators online
				//This will also not run if it is a cab request
				priority_value := fsm.Calculate_priority_value(button_event)
				connectivity.New_order(button_event, priority_value)
			} else {
				// If elevator do not see any other elevators are online. Do the request selfe
				fmt.Println("No other online elevators or a cab call. Take order")
				fsm.Fsm_onRequestButtonPress(button_event.Floor, button_event.Button)
			}

		case a := <-drv_floors: // Hvis det kommer en etasje (int) fra chanelen drv_floors
			fmt.Println("Floor event")
			fmt.Printf("%+v\n", a)
			if a != -1 && a != prev_floor { // Hvis heisen er i en etasje og etasjen er ulik den forrige etasjen
				fsm.Fsm_onFloorArrival(a)
			}
			prev_floor = a

		case a := <-timerTimeoutChan: // Hvis det kommer en bool, True, fra chanelen timerTimeoutChan
			if a {
				fmt.Println("Door timeout")
				fsm.TimerStop()
				fsm.Fsm_onDoorTimeout()
			}

		case a := <-drv_obstr:
			fmt.Println("Obstruction event toggle")
			fsm.SetObsructionStatus(a)
			fsm.TimerStart(3)

		case <-world_view_send_ticker:
			//fmt.Println("Sending world view")
			connectivity.Send_world_view()

			send_world_view_package := connectivity.New_Worldview_package(connectivity.ID, fsm.GetElevatorStruct())
			connectivity.PrintOrderRequest(send_world_view_package.Order_requeset)
			connectivity.PrintOrderRequest(send_world_view_package.Order_response)

			//connectivity.PrintIsOnline()

			//case world_view := <-TCP_receive_channel:
			//fmt.Println("World view reseved, PC:", world_view.Elevator_ID, "\n")
			//fmt.Println("\n\n")
			//time.Sleep(500 * time.Duration(inputPollRateMs))

		case received_world_view := <-TCP_receive_channel:
			//fmt.Println("World view reseved, PC:", received_world_view.Elevator_ID)

			//storing worldview
			//connectivity.Store_worldview(received_world_view.Elevator_ID, received_world_view)

			if received_world_view.Order_bool {
				fmt.Println("Order receved")
				fsm.Fsm_onRequestButtonPress(received_world_view.Order.Floor, received_world_view.Order.Button)
			}

			//priority_value := fsm.Calculate_priority_value(received_world_view.)
			connectivity.Receved_order_requests(received_world_view.Order_requeset, received_world_view.Elevator_ID) //Mulig vi kan flytte denne inn i conneciton pakka
			//fmt.Println("Receved_order_response, id: ", received_world_view.Elevator_ID)
			connectivity.Receved_order_response(received_world_view.Order_response)

		case received_order := <-order_to_send_chan:
			if !connectivity.SendOrderToSpesificElevator(received_order) {
				fmt.Println("Failed to send order. taking it selfe")
				fsm.Fsm_onRequestButtonPress(received_order.Order.Floor, received_order.Order.Button)
			}
		}

	}

}
