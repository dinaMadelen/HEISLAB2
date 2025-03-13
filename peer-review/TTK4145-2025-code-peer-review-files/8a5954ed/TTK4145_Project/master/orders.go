package master

import (
	"network"
	"elevio"
	"datatype"
	"config"
	"math"
	"log"
	"color"
)

func (m *Master) Distribute_order(floor int, button elevio.ButtonType, fromID string) {
	// Cab order -> send to the same client
	if button == elevio.BT_Cab {
		m.Dispatch_order(floor, button, fromID, fromID)
	// Hall order -> distribute to closest elevator with least orders
	} else {
		clientID := m.Find_closest_elevator(floor)
		m.Dispatch_order(floor, button, fromID, clientID)
	}
}

// Send order to client and update queue
func (m *Master) Dispatch_order(floor int, button elevio.ButtonType, fromID string, toID string) {
	addr := m.Client_list.Get_addr_from_id(toID)
	m.Queue.Update_send_to(floor, button, fromID, toID)
	m.Sync_to_backup()
	network.Send_message(m.Client_list.Get(addr).Conn,
		datatype.HeaderType.OrderReceived,
		datatype.DataPayload{OrderFloor: floor, OrderButton: button})
	m.Client_list.Get(addr).Active_orders++
	m.Client_list.Get(addr).Task_timer.Reset(config.Task_period)
}

func (m *Master) Find_closest_elevator(floor int) string {
	closestId := ""
	minValue := math.MaxInt

	if m.Client_list.Length() > 0 {
		for _, client := range m.Client_list.Clients {
			if client.Client_type == datatype.ClientType.Elevator {
				distance := int(math.Abs(float64(client.Current_floor - floor))) + client.Active_orders * 2
				if distance < minValue {
					minValue = distance
					closestId = client.ID
				}
			}
		}
	}

	if closestId == "" {
		log.Print(color.Red + "No elevators available \n" + color.Reset)
	}

	return closestId
}

func (m *Master) Redistribute_client_orders(id string) {
	for _, order := range m.Queue.Orders {
		if order.SentTo == id {
			// Dont redistribute cab order
			if order.Button == elevio.BT_Cab {
				m.Queue.Update_send_to(order.Floor, order.Button, order.ReceivedFrom, "")
			// Redistribute hall order
			} else {
				m.Distribute_order(order.Floor, order.Button, order.ReceivedFrom)
			}
		}
	}
}

func (m *Master) Resend_queue(id string) {
	for _, order := range m.Queue.Orders {
		if order.SentTo == id {
			addr := m.Client_list.Get_addr_from_id(order.SentTo)
			network.Send_message(m.Client_list.Get(addr).Conn,
				datatype.HeaderType.OrderReceived,
				datatype.DataPayload{OrderFloor: order.Floor, OrderButton: order.Button, ID: order.ReceivedFrom})
			m.Client_list.Get(addr).Active_orders++
		}
	}
}