package elevator

import (
	"network"
	"datatype"
	"elevio"
	"config"
	"log"
	"master"
)

func (e *Elevator) Handle_floor_signal(floor int) {
	e.Current_floor = floor
	elevio.SetFloorIndicator(floor)

	if e.Connected {
		network.Send_message(e.Conn,
							 datatype.HeaderType.UpdateFloor,
							 datatype.DataPayload{CurrentFloor: floor})
	}
}

func (e *Elevator) Handle_button_signal(btn elevio.ButtonEvent) {
	if e.Connected {
		network.Send_message(e.Conn,
							 datatype.HeaderType.OrderReceived,
							 datatype.DataPayload{OrderFloor: btn.Floor,
												  OrderButton: btn.Button,
												  ID:          e.ID})
	} else if !e.Connected && btn.Button == elevio.BT_Cab {
		e.Add_order(datatype.Queue_element{Floor: btn.Floor,
										   Button: btn.Button})
	}
}

func (e *Elevator) Handle_obstruction_signal(obs bool) {
	e.Obstruction = obs
	if e.Connected {
		network.Send_message(e.Conn,
							 datatype.HeaderType.UpdateObstruction,
							 datatype.DataPayload{Obstruction: obs})
	}
}

func (e *Elevator) Handle_connection_loss() {
	e.Connected = false
	e.Remove_non_cab_orders()
	e.Reconnect_timer.Reset(config.Reconnect_delay)
	e.Current_state = State.Undefined
}

func (e *Elevator) Handle_new_message(message network.Message) {
	switch message.Header {
	case datatype.HeaderType.OrderReceived:
		log.Print("Order received: \n")
		e.Add_order(datatype.Queue_element{Floor: message.Payload.OrderFloor,
										   Button: message.Payload.OrderButton})

	case datatype.HeaderType.StartBackup:
		log.Print("Starting backup: \n")
		master.Start_backup()
	}
}