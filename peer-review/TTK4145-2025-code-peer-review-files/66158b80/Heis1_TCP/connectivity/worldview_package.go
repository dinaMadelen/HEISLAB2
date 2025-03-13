package connectivity

import (
	"Driver-go/elevio"
	"Driver-go/fsm"
	"fmt"
)

type Worldview_package struct {
	Elevator_ID    int
	Elevator       fsm.Elevator //refrence
	Order_requeset []OrderRequests
	Order_response []OrderRequests
	Order          elevio.ButtonEvent
	Order_bool     bool
}

func New_Worldview_package(elevator_id int, elevator_ fsm.Elevator) Worldview_package {
	pending_orders := Get_pending_orders()
	responses := Get_order_respons()
	return Worldview_package{
		Elevator_ID:    elevator_id,
		Elevator:       elevator_,
		Order_requeset: pending_orders,
		Order_response: responses,
		Order_bool:     false,
	}
}

func PrintWorldview(world_view Worldview_package) {
	fmt.Print("WorldView")
	fmt.Println("Elevator ID: ", world_view.Elevator_ID)

}
