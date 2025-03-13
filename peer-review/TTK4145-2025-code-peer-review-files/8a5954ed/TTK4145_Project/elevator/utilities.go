package elevator

import (
	"color"
	"config"
	"datatype"
	"elevio"
	"log"
	"net"
	"network"
)

func Reset_lamps_at_floor(floor int) {
	elevio.SetButtonLamp(elevio.BT_Cab, floor, false)
	elevio.SetButtonLamp(elevio.BT_HallUp, floor, false)
	elevio.SetButtonLamp(elevio.BT_HallDown, floor, false)
}

func Reset_all_lamps(floor_num int) {
	for floor := 0; floor < floor_num; floor++ {
		Reset_lamps_at_floor(floor)
	}
}

// Connect elevator to server and send client infomation
func (e *Elevator) Connect_elevator(msgChan chan<- network.Message, connLoss chan<- *net.TCPConn) {
	for {
		select {
		case <-e.Reconnect_timer.C:
			e.Reconnect_timer.Stop()
			serverAddr := network.Find_server_address()
			if serverAddr != "" {
				e.Conn = network.Connect_to_server(serverAddr)
				e.Connected = true
				log.Printf(color.Green + "Connected to server: %s\n" + color.Reset, network.Get_addr_from_conn(e.Conn))
				network.Send_message(e.Conn,
									 datatype.HeaderType.ClientInfo,
									 datatype.DataPayload{ClientType:   datatype.ClientType.Elevator,
														  ID:           e.ID,
														  CurrentFloor: e.Current_floor,
														  Obstruction:  e.Obstruction})
				go network.Listen_for_message(e.Conn, msgChan, connLoss)
				// Resend all finished orders
				e.Resend_finished_orders()
			} else {
				e.Reconnect_timer.Reset(config.Reconnect_delay)
			}
		default:
		}
	}
}
