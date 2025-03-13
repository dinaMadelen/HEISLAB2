package master

import (
	"config"
	"datatype"
	"fmt"
	"log"
	"net"
	"network"
	"time"
	"color"
	"strings"
)

type Client struct {
	Conn			*net.TCPConn
	Client_type		string
	ID 				string
	Current_floor	int
	Obstruction		bool
	Active_orders	int
	Task_timer		*time.Timer
}

// Mapped by client address (ip:port)
type Client_list struct {
	Clients map[string]*Client
}

func Create_client_list() *Client_list {
	return &Client_list{
		Clients: make(map[string]*Client),
	}
}

func (cl *Client_list) Get(addr string) *Client {
	client, exists := cl.Clients[addr]
	if exists {
		return client
	}
	return nil
}

func (cl *Client_list) Add(addr string, conn *net.TCPConn) {
	cl.Clients[addr] = &Client{
		Conn:          conn,
		Client_type:   datatype.ClientType.Unknown,
		Current_floor: -1,
		Task_timer:    time.NewTimer(config.Task_period),
		Obstruction:   false,
	}
	cl.Clients[addr].Task_timer.Stop()
}

func (cl *Client_list) Remove(addr string) {
	_, exists := cl.Clients[addr]
	if exists {
		delete(cl.Clients, addr)
	}
}

func (cl *Client_list) Update(addr string, clientType *string, ID *string, floor *int, obstruction *bool) {
	if client, exists := cl.Clients[addr]; exists {
		if clientType != nil {
			client.Client_type = *clientType
		}
		if ID != nil {
			client.ID = *ID
		}
		if floor != nil {
			client.Current_floor = *floor
		}
		if obstruction != nil {
			client.Obstruction = *obstruction
		}
	}
}

func (cl *Client_list) Length() int {
	return len(cl.Clients)
}

func (cl *Client_list) Choose_backup_host(server_addr string) *net.TCPConn {
	for _, client := range cl.Clients {
		addr := network.Get_addr_from_conn(client.Conn)
		if strings.Split(addr, ":")[0] != strings.Split(server_addr, ":")[0] {
			return client.Conn
		}
	}
	return nil
}

func (cl *Client_list) Print() {
	fmt.Print("Client list: \n")
	for addr, client := range cl.Clients {
		if client.Client_type == datatype.ClientType.Elevator {
			fmt.Printf("Addres: %s, type: %s, ID: %s, AO: %d, Obs: %t, Floor: %d \n",
				addr, client.Client_type, client.ID, client.Active_orders, client.Obstruction, client.Current_floor)
		} else {
			fmt.Printf("Addres: %s, type: %s, ID: %s \n", addr, client.Client_type, client.ID)
		}
	}
	fmt.Println()
}

func (cl *Client_list) Get_addr_from_id(id string) string {
	for _, client := range cl.Clients {
		if client.ID == id {
			return network.Get_addr_from_conn(client.Conn)
		}
	}
	log.Printf(color.Red + "Cant find address for id: %s" + color.Reset, id)
	return ""
}

// Checks task timer for each client
func (cl *Client_list) Timer_handler(taskTimeout chan<- string) {
	for {
		time.Sleep(500 * time.Millisecond)
		for addr, client := range cl.Clients {
			if client.Task_timer != nil {
				select {
				case <-client.Task_timer.C:
					taskTimeout <- addr

				default:
				}
			}
		}
	}
}
