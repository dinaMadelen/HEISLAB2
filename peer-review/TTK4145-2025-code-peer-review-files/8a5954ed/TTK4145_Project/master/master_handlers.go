package master

import (
	"config"
	"datatype"
	"log"
	"net"
	"network"
	"os"
	"color"
)

func (m *Master) Handle_disconnect(conn *net.TCPConn) {
	addr := network.Get_addr_from_conn(conn)

	// If backup disconnects
	if m.Client_list.Get(addr).Client_type == datatype.ClientType.Backup {
		m.Client_list.Remove(addr)
		m.Update_backup_info()

		if network.Ping_google() {
			m.Request_backup()
		} else {
			os.Exit(0)
		}

	// If elevator disconnects
	} else if m.Client_list.Get(addr).Client_type == datatype.ClientType.Elevator {
		client_id := m.Client_list.Get(addr).ID
		m.Client_list.Remove(addr)
		m.Redistribute_client_orders(client_id)

	// If unknown client disconnects
	} else {
		m.Client_list.Remove(addr)
	}
}

func (m *Master) Handle_new_messages(msg network.Message) {
	switch msg.Header {
	// Floor update
	case datatype.HeaderType.UpdateFloor:
		m.Client_list.Update(msg.Addr, nil, nil, &msg.Payload.CurrentFloor, nil)
		m.Client_list.Print()

	// Obstruction update
	case datatype.HeaderType.UpdateObstruction:
		m.Client_list.Update(msg.Addr, nil, nil, nil, &msg.Payload.Obstruction)
		m.Client_list.Print()

	// Client info
	case datatype.HeaderType.ClientInfo:
		m.Handle_client_info(msg)

	// Order received
	case datatype.HeaderType.OrderReceived:
		m.Handle_recieved_order(msg)

	// Order fulfilled
	case datatype.HeaderType.OrderFulfilled:
		m.Handle_fulfilled_orders(msg)

	// Sync confirmation
	case datatype.HeaderType.SyncConfirmation:
		m.Handle_sync_confirmation(msg)
	}
}

func (m *Master) Handle_recieved_order(msg network.Message) {
	order := datatype.Master_queue_element{Floor: msg.Payload.OrderFloor,
										   Button:       msg.Payload.OrderButton,
										   ReceivedFrom: msg.Payload.ID,
										   SentTo:       ""}

	// Check if order is not an duplicate
	if !m.Queue.Check_for_duplicate(order) {
		m.Queue.Add(order)
		m.Sync_to_backup()
		m.Distribute_order(msg.Payload.OrderFloor, msg.Payload.OrderButton, msg.Payload.ID)
		m.Sync_to_backup()
	} 
}

func (m *Master) Handle_client_info(msg network.Message) {
	m.Client_list.Update(msg.Addr, &msg.Payload.ClientType, &msg.Payload.ID, &msg.Payload.CurrentFloor, &msg.Payload.Obstruction)
	m.Client_list.Print()

	if msg.Payload.ClientType == datatype.ClientType.Backup {
		m.Backup_check_timer.Stop()
		m.Update_backup_info()
		m.Sync_to_backup()
	}
	// Resend queue after backup takes over as master
	if m.Queue.Length() > 0 {
		m.Resend_queue(msg.Payload.ID)
	}
}

func (m *Master) Handle_fulfilled_orders(msg network.Message) {
	m.Queue.Remove(msg.Payload.OrderFloor, msg.Payload.OrderButton, msg.Payload.ID)
	if m.Client_list.Get(msg.Addr).Active_orders > 0 {
		m.Client_list.Get(msg.Addr).Active_orders--
	}
	
	if m.Client_list.Get(msg.Addr).Active_orders > 0 {
		m.Client_list.Get(msg.Addr).Task_timer.Reset(config.Task_period + config.Door_open_duration)
	} else {
		m.Client_list.Get(msg.Addr).Task_timer.Stop()
	}
	m.Sync_to_backup()
	m.Queue.Print()
}

// Check if confirmation has the same queue as master, if not resend queue
func (m *Master) Handle_sync_confirmation(msg network.Message) {
	if len(msg.Payload.Queue) != m.Queue.Length() {
		m.Sync_to_backup()
		return
	}

	for i := range m.Queue.Orders {
		// Check for difference
		if m.Queue.Orders[i] != msg.Payload.Queue[i] {
			m.Sync_to_backup()
			return
		}
	}

	log.Print(color.Green + "Queue synced successfully \n" + color.Reset)
}

func (m *Master) Handle_task_timeout(addr string) {
	if m.Client_list.Get(addr).Obstruction {
		m.Client_list.Get(addr).Task_timer.Reset(config.Task_period + config.Door_open_duration)
	} else {
		log.Printf(color.Red + "Client, %s, used too long to fulfill the task. \n" + color.Reset, addr)
		client_id := m.Client_list.Get(addr).ID
		m.Client_list.Get(addr).Conn.Close()
		m.Redistribute_client_orders(client_id)
	}
}

func (m *Master) Timer_manager() {
	for {
		select {
		// Check if there are not distributed orders
		case <-m.Distribution_timer.C:
			m.Handle_distribution_timer()

		// Check if backup exists
		case <-m.Backup_check_timer.C:
			if !m.Backup_exists {
				m.Request_backup()
			}
		}
	}
}

func (m *Master) Handle_distribution_timer() {
	for _, order := range m.Queue.Orders {
		if order.SentTo == "" {
			m.Distribute_order(order.Floor, order.Button, order.ReceivedFrom)
		}
	}
	m.Distribution_timer.Reset(config.Distribution_delay)
}